use core::{
    cmp::min,
    net::{Ipv4Addr, SocketAddrV4},
};

use log::info;
use smoltcp::{
    iface::SocketHandle,
    socket::{
        dns::{self, GetQueryResultError},
        tcp::{self, State},
    },
    wire::{DnsQueryType, Ipv4Address},
};

use as_hostcall::{
    socket::{SmoltcpError, SmoltcpResult},
    types::{Size, SockFd},
};
#[cfg(feature = "log")]
use as_std::println;

use crate::{acquire_iface, acquire_sockets, from_sockfd, iface_poll, to_sockfd, try_phy_wait};

#[no_mangle]
pub fn addrinfo(name: &str) -> SmoltcpResult<Ipv4Addr> {
    // println!("Device raw_fd is {}", fd);
    let mut iface = acquire_iface()?;

    let (dns_handle, query) = {
        let servers = &[Ipv4Address::new(114, 114, 114, 114).into()];
        let dns_socket = dns::Socket::new(servers, vec![]);

        let mut sockets = acquire_sockets()?;

        let dns_handle: SocketHandle = sockets.add(dns_socket);
        let dns_socket = sockets.get_mut::<dns::Socket>(dns_handle);
        let query = dns_socket
            .start_query(iface.context(), name, DnsQueryType::A)
            .map_err(|e| {
                println!("query err: {}", e);
                SmoltcpError::DNSQueryFailed
            })?;

        (dns_handle, query)
    };

    loop {
        let mut sockets = acquire_sockets()?;
        let timestamp = iface_poll(&mut iface, &mut sockets);

        let dns_socket = sockets.get_mut::<dns::Socket>(dns_handle);
        match dns_socket.get_query_result(query) {
            Ok(addrs) => {
                let addr = match addrs.first().ok_or(SmoltcpError::DNSQueryFailed)? {
                    smoltcp::wire::IpAddress::Ipv4(ipv4) => ipv4.0,
                };
                sockets.remove(dns_handle);
                return Ok(Ipv4Addr::from(addr));
            }
            Err(GetQueryResultError::Pending) => {} // not done yet
            Err(e) => Err(SmoltcpError::SmoltcpErr(e.to_string()))?,
        }

        try_phy_wait(timestamp, &mut iface, &mut sockets)?
    }
}

#[no_mangle]
pub fn smol_connect(sockaddr: SocketAddrV4) -> SmoltcpResult<SockFd> {
    info!("smol_connect");
    let mut iface = acquire_iface()?;
    let mut sockets = acquire_sockets()?;

    let tcp_handle = {
        let tcp_rx_buffer = tcp::SocketBuffer::new(vec![0; 1024]);
        let tcp_tx_buffer = tcp::SocketBuffer::new(vec![0; 1024]);
        let socket = tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer);
        sockets.add(socket)
    };

    // let timestamp = Instant::now();
    // iface.poll(timestamp, device, &mut sockets);

    let tcp_socket: &mut tcp::Socket = sockets.get_mut::<tcp::Socket>(tcp_handle);
    if tcp_socket.is_active() {
        // bad state.
        return Err(SmoltcpError::BadTCPState(
            "is_active".to_owned(),
            tcp_socket.state().to_string(),
        ));
    };

    let cx = iface.context();
    let local_port = 49152 + rand::random::<u16>() % 16384;
    tcp_socket
        .connect(cx, sockaddr, local_port)
        .map_err(|e| SmoltcpError::SmoltcpErr(e.to_string()))?;

    Ok(to_sockfd(tcp_handle))
}

#[no_mangle]
pub fn smol_send(handle: SockFd, data: &[u8]) -> SmoltcpResult<()> {
    // println!("smol_send, handle={}", handle);
    let mut iface = acquire_iface()?;
    let mut sockets = acquire_sockets()?;
    let mut cursor = 0;

    loop {
        let timestamp = iface_poll(&mut iface, &mut sockets);

        let socket = sockets.get_mut::<tcp::Socket>(from_sockfd(handle));

        // send_slice doesn't mean it's actually sent
        if socket.may_send() {
            // println!(
            //     "sock_{} remote={:?}, send slice: {:?},",
            //     handle,
            //     socket.remote_endpoint(),
            //     String::from_utf8_lossy(&data[cursor..])
            // );
            cursor += socket
                .send_slice(&data[cursor..])
                .map_err(|e| SmoltcpError::SmoltcpErr(e.to_string()))?;
        }

        if cursor == data.len() {
            return Ok(());
        }

        try_phy_wait(timestamp, &mut iface, &mut sockets)?
    }
}

#[no_mangle]
pub fn smol_recv(handle: SockFd, buf: &mut [u8]) -> SmoltcpResult<Size> {
    println!("smol_recv");
    let mut iface = acquire_iface()?;
    let mut sockets = acquire_sockets()?;

    let mut cursor = 0;
    let mut freesize = buf.len();

    // should return data if buffer is not empty. otherwise will block until closed TCP.
    loop {
        let timestamp = iface_poll(&mut iface, &mut sockets);
        let tcp_socket = sockets.get_mut::<tcp::Socket>(from_sockfd(handle));

        // println!("wait for recv, status={}", tcp_socket.state());
        if cursor != 0 || !tcp_socket.may_recv() || tcp_socket.state() == State::CloseWait {
            break;
        } else if tcp_socket.can_recv() {
            tcp_socket
                .recv(|data| {
                    // avoid to overflow the buffer.
                    let size = min(buf.len() - cursor, data.len());
                    buf[cursor..(cursor + size)].copy_from_slice(&data[0..size]);
                    cursor += size;
                    freesize = buf.len() - cursor;
                    // println!("read size={}, free size={}", size, freesize);
                    (size, ())
                })
                .map_err(|e| SmoltcpError::SmoltcpErr(e.to_string()))?
        };
        try_phy_wait(timestamp, &mut iface, &mut sockets)?
    }

    Ok(cursor)
}

#[no_mangle]
pub fn smol_close(handle: SockFd) -> SmoltcpResult<()> {
    // println!("smol_close");
    let mut iface = acquire_iface()?;

    let handle = from_sockfd(handle);
    let mut has_close = false;

    loop {
        let mut sockets = acquire_sockets()?;
        let timestamp = iface_poll(&mut iface, &mut sockets);

        let tcp_socket = sockets.get_mut::<tcp::Socket>(handle);
        if !has_close && tcp_socket.send_queue() == 0 {
            // println!(
            //     "send queue is empty, can close sock: {}, state: {}",
            //     handle,
            //     tcp_socket.state()
            // );

            // Should not remove socket from sockets. Because smoltcp will
            // update state of tcp connect by sockets.
            tcp_socket.close();
            has_close = true;
        }

        // println!("{:?}", tcp_socket.state());
        if !tcp_socket.is_open() {
            break;
        }

        try_phy_wait(timestamp, &mut iface, &mut sockets)?
    }

    Ok(())
}

#[no_mangle]
pub fn smol_bind(addr: SocketAddrV4) -> SmoltcpResult<SockFd> {
    let mut iface = acquire_iface()?;
    let mut sockets = acquire_sockets()?;

    let tcp_handle = {
        let tcp_rx_buffer = tcp::SocketBuffer::new(vec![0; 64]);
        let tcp_tx_buffer = tcp::SocketBuffer::new(vec![0; 128]);
        let socket = tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer);
        sockets.add(socket)
    };

    iface_poll(&mut iface, &mut sockets);

    let socket = sockets.get_mut::<tcp::Socket>(tcp_handle);
    if socket.is_open() {
        Err(SmoltcpError::BadTCPState(
            "is_open".to_owned(),
            socket.state().to_string(),
        ))?
    };
    socket
        .listen(addr.port())
        .map_err(|e| SmoltcpError::SmoltcpErr(e.to_string()))?;

    Ok(to_sockfd(tcp_handle))
}

#[no_mangle]
pub fn smol_accept(handle: SockFd) -> SmoltcpResult<SockFd> {
    let mut iface = acquire_iface()?;
    let mut sockets = acquire_sockets()?;

    let conned_sock = loop {
        let timestamp = iface_poll(&mut iface, &mut sockets);
        let listened_sock = sockets.get_mut::<tcp::Socket>(from_sockfd(handle));

        // println!("{:?}", listened_sock.state());
        if listened_sock.state() != State::Listen {
            break listened_sock;
        }

        try_phy_wait(timestamp, &mut iface, &mut sockets)?
    };

    // println!("sock state is {:?}", sock.state());
    // println!("remote is {}", conned_sock.remote_endpoint().unwrap());
    let local_port = conned_sock
        .local_endpoint()
        .ok_or(SmoltcpError::NoLocalEndpoint)?
        .port;

    drop(iface);
    drop(sockets);

    smol_bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), local_port))
}
