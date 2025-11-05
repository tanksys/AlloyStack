#![feature(ip_in_core)]

mod drop_tap;
// mod logs;
mod setup_tap;

use core::net::Ipv4Addr;
use std::{
    os::fd::AsRawFd,
    sync::{Mutex, MutexGuard},
};

use lazy_static::lazy_static;
use log::{info, LevelFilter};
use smoltcp::{
    iface::{Config, Interface, SocketHandle, SocketSet},
    phy::{wait as phy_wait, Device, Medium, TunTapInterface},
    time::Instant,
    wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address},
};

use crate::setup_tap::exec_tap_setup;
use as_hostcall::{
    socket::{SmoltcpError, SmoltcpResult},
    types::{NetdevName, SockFd},
};
use as_std::init_context;

pub mod apis;
pub mod logs;

thread_local! {
    static DEVICE: Mutex<TunTapInterface> = {
        log::set_logger(&logs::LOGGER).expect("fail to init log");
        #[cfg(feature = "log")] {
            log::set_max_level(LevelFilter::Trace);
        }
        #[cfg(not(feature = "log"))] {
            log::set_max_level(LevelFilter::Off);
        }
        info!("begin init DEVICE");
        let mut netdev_name = NETDEV_NAME.lock().unwrap();
        if netdev_name.is_none() {
            *netdev_name = Some(NetdevName{
                name: format!("tap-{}", init_context::isolation_ctx().isol_id),
                subnet: Ipv4Addr::new(192, 168, 69, 0),
                mask: 24
            });
        };

        exec_tap_setup(netdev_name.as_ref().unwrap()).expect("setup tap device failed.");

        info!("finish init DEVICE");
        Mutex::from(TunTapInterface::new(&netdev_name.as_ref().unwrap().name, Medium::Ethernet).unwrap())
    };
}

lazy_static! {
    /// NETDEV_NAME is tap device name, such as tap-42, and also index the ip
    /// address that libos should use.
    ///
    /// Here use lazy_static because device name and ip address are allocated
    /// when runtime.
    ///
    /// Here use Option<> because the `socket` module may never be used so in
    /// this case shouldn't ask for a NetdevName.
    pub static ref NETDEV_NAME: Mutex<Option<NetdevName>> = Mutex::default();
    static ref TAP_RAW_FD: i32 =  DEVICE.with(|device| device.lock().unwrap().as_raw_fd());
    static ref SOCKETS: Mutex<SocketSet<'static>> = Mutex::new(SocketSet::new(vec![]));

    static ref IFACE: Mutex<Interface> = {
        // logs::setup_logging("");

        let mut iface = DEVICE.with(|device_tls| {
            let mut device = device_tls.lock().unwrap();
            let mut config = match device.capabilities().medium {
                Medium::Ethernet => {
                    Config::new(EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]).into())
                }
                Medium::Ip => Config::new(smoltcp::wire::HardwareAddress::Ip),
                // Medium::Ieee802154 => todo!(),
            };

            config.random_seed = rand::random();
            Interface::new(config, &mut (*device), Instant::now())
        });

        iface.update_ip_addrs(|ip_addrs| {
            ip_addrs
                .push(IpCidr::new(
                    // IpAddress::v4(192, 168, 69, init_context::isolation_ctx().isol_id as u8),
                    IpAddress::v4(192, 168, 69, 1),
                    24,
                ))
                .unwrap();
        });
        // The default ipv4 route is equal to ip address of tap device. (Look scripts/make_tap.sh).
        iface
            .routes_mut()
            .add_default_ipv4_route(Ipv4Address::new(192, 168, 69, 100))
            .unwrap();

        Mutex::new(iface)
    };
}

fn get_tap_raw_fd() -> i32 {
    *TAP_RAW_FD
}

fn from_sockfd(handle: SockFd) -> SocketHandle {
    unsafe { core::mem::transmute(handle as usize) }
}

fn to_sockfd(handle: SocketHandle) -> SockFd {
    let fd: usize = unsafe { core::mem::transmute(handle) };
    fd as SockFd
}

fn iface_poll(iface: &mut Interface, sockets: &mut SocketSet) -> Instant {
    let timestamp = Instant::now();

    DEVICE.with(|device_tls| {
        let mut device = device_tls.lock().unwrap();
        iface.poll(timestamp, &mut (*device), sockets)
    });

    timestamp
}

fn try_phy_wait(
    timestamp: Instant,
    iface: &mut Interface,
    sockets: &mut SocketSet,
) -> SmoltcpResult<()> {
    phy_wait(get_tap_raw_fd(), iface.poll_delay(timestamp, sockets))
        .map_err(|e| SmoltcpError::HostIOErr(e.to_string()))
}

fn acquire_sockets() -> SmoltcpResult<MutexGuard<'static, SocketSet<'static>>> {
    SOCKETS
        .lock()
        .map_err(|e| SmoltcpError::AcquireLockErr("SOCKETS".to_owned(), e.to_string()))
}

fn acquire_iface() -> SmoltcpResult<MutexGuard<'static, Interface>> {
    IFACE
        .lock()
        .map_err(|e| SmoltcpError::AcquireLockErr("IFACE".to_owned(), e.to_string()))
}
