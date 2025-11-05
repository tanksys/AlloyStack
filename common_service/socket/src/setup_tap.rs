use core::net::Ipv4Addr;
use std::{
    env,
    io::{Read, Write},
    process::{Command, Stdio},
};

use as_hostcall::types::NetdevName;

fn gen_sudo_command() -> Command {
    let mut comd = Command::new("sudo");
    comd.arg("-S")
        // .arg("-n")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    comd
}

fn gen_tap_setup(netdev_name: &NetdevName) -> Vec<Command> {
    let mut commands = vec![];
    let subnet_mask_str = format!("{}/{}", netdev_name.subnet, netdev_name.mask);
    let ipv4_mask_str = {
        let ipaddr = {
            let mut ipaddr = netdev_name.subnet.octets();
            ipaddr[3] = 100;
            Ipv4Addr::from(ipaddr)
        };
        format!("{}/{}", ipaddr, netdev_name.mask)
    };

    // sudo -S ip tuntap add name tap0 mode tap user $USER
    commands.push({
        let mut comd = gen_sudo_command();
        comd.arg("ip")
            .arg("tuntap")
            .arg("add")
            .arg("name")
            .arg(&netdev_name.name)
            .arg("mode")
            .arg("tap")
            .arg("user")
            .arg(env::var("USER").unwrap());
        comd
    });

    // sudo -S ip link set tap0 up
    commands.push({
        let mut comd = gen_sudo_command();
        comd.arg("ip")
            .arg("link")
            .arg("set")
            .arg(&netdev_name.name)
            .arg("up");
        comd
    });

    // sudo -S ip addr add 192.168.69.100/24 dev tap0
    commands.push({
        let mut comd = gen_sudo_command();
        comd.arg("ip")
            .arg("addr")
            .arg("add")
            .arg(&ipv4_mask_str)
            .arg("dev")
            .arg(&netdev_name.name);
        comd
    });

    // sudo -S iptables -t nat -A POSTROUTING -s 192.168.69.0/24 -j MASQUERADE
    commands.push({
        let mut comd = gen_sudo_command();
        comd.arg("iptables")
            .arg("-t")
            .arg("nat")
            .arg("-A")
            .arg("POSTROUTING")
            .arg("-s")
            .arg(&subnet_mask_str)
            .arg("-j")
            .arg("MASQUERADE");
        comd
    });

    // sudo -S iptables -A FORWARD -i tap0 -s 192.168.69.0/24 -j ACCEPT
    commands.push({
        let mut comd = gen_sudo_command();
        comd.arg("iptables")
            .arg("-A")
            .arg("FORWARD")
            .arg("-i")
            .arg(&netdev_name.name)
            .arg("-s")
            .arg(&subnet_mask_str)
            .arg("-j")
            .arg("ACCEPT");
        comd
    });

    // sudo -S iptables -A FORWARD -o tap0 -d 192.168.69.0/24 -j ACCEPT
    commands.push({
        let mut comd = gen_sudo_command();
        comd.arg("iptables")
            .arg("-A")
            .arg("FORWARD")
            .arg("-o")
            .arg(&netdev_name.name)
            .arg("-d")
            .arg(&subnet_mask_str)
            .arg("-j")
            .arg("ACCEPT");
        comd
    });

    commands
}

#[test]
fn test_gen_tap_setup() {
    let result = gen_tap_setup(&NetdevName {
        name: "tap0".to_string(),
        subnet: Ipv4Addr::new(192, 168, 69, 0),
        mask: 24,
    });
    assert_eq!(
        format!("{:?}", result[0]),
        format!(
            "\"sudo\" \"-S\" \"-n\" \"ip\" \"tuntap\" \"add\" \"name\" \"tap0\" \"mode\" \"tap\" \"user\" \"{}\"",
            env::var("USER").unwrap()
        )
    );

    assert_eq!(
        format!("{:?}", result[1]),
        "\"sudo\" \"-S\" \"-n\" \"ip\" \"link\" \"set\" \"tap0\" \"up\""
    );

    assert_eq!(
        format!("{:?}", result[2]),
        "\"sudo\" \"-S\" \"-n\" \"ip\" \"addr\" \"add\" \"192.168.69.100/24\" \"dev\" \"tap0\""
    );

    assert_eq!(
        format!("{:?}", result[3]),
        "\"sudo\" \"-S\" \"-n\" \"iptables\" \"-t\" \"nat\" \"-A\" \"POSTROUTING\" \"-s\" \"192.168.69.0/24\" \"-j\" \"MASQUERADE\""
    );

    assert_eq!(
        format!("{:?}", result[4]),
        "\"sudo\" \"-S\" \"-n\" \"iptables\" \"-A\" \"FORWARD\" \"-i\" \"tap0\" \"-s\" \"192.168.69.0/24\" \"-j\" \"ACCEPT\""
    );

    assert_eq!(
        format!("{:?}", result[5]),
        "\"sudo\" \"-S\" \"-n\" \"iptables\" \"-A\" \"FORWARD\" \"-o\" \"tap0\" \"-d\" \"192.168.69.0/24\" \"-j\" \"ACCEPT\""
    );
}

fn exec_sudo_commands(commands: Vec<Command>) -> Result<(), String> {
    // println!(
    //     "exec_sudo_commands, pid={}, thread_id={:?}",
    //     std::process::id(),
    //     std::thread::current().id()
    // );
    let mut passwd = env::var("SUDO_PASSWD").expect("can't get root permission");
    if !passwd.ends_with('\n') {
        passwd += "\n";
    }

    // println!("the sudo passwd: {}", passwd);
    // assert_eq!(passwd.as_bytes(), b"cptbtptp\n");
    for mut comd in commands {
        // println!("The next cmd: {:?}", comd);
        let mut child = comd
            .spawn()
            .unwrap_or_else(|_| panic!("exec: {:?} failed", comd));
        child
            .stdin
            .as_mut()
            .ok_or("Failed")
            .unwrap()
            .write_all(passwd.as_bytes())
            .unwrap();
        let result = child.wait().unwrap();
        if result.success() {
            continue;
        };
        return Err(format!(
            "exec command failed, command: {:?}, stderr: {}, stdout: {}",
            comd,
            {
                let mut buf = String::new();
                child.stderr.unwrap().read_to_string(&mut buf).unwrap();
                buf
            },
            {
                let mut buf = String::new();
                child.stdout.unwrap().read_to_string(&mut buf).unwrap();
                buf
            }
        ));
    }
    Ok(())
}

pub fn exec_tap_setup(netdev_name: &NetdevName) -> Result<(), String> {
    let commands = gen_tap_setup(netdev_name);
    exec_sudo_commands(commands)
}

fn gen_tap_cleanup(netdev_name: &NetdevName) -> Vec<Command> {
    let mut commands = vec![];

    // ip link set tap0 down
    commands.push({
        let mut comd = gen_sudo_command();
        comd.arg("ip")
            .arg("link")
            .arg("set")
            .arg(&netdev_name.name)
            .arg("down");

        comd
    });

    // ip tuntap del dev tap0 mode tap
    commands.push({
        let mut comd = gen_sudo_command();
        comd.arg("ip")
            .arg("tuntap")
            .arg("del")
            .arg("dev")
            .arg(&netdev_name.name)
            .arg("mode")
            .arg("tap");

        comd
    });

    commands
}

#[test]
fn test_gen_tap_cleanup() {
    let result = gen_tap_cleanup(&NetdevName {
        name: "tap0".to_string(),
        subnet: Ipv4Addr::new(192, 168, 69, 0),
        mask: 24,
    });

    assert_eq!(
        format!("{:?}", result[0]),
        format!(
            "\"sudo\" \"-S\" \"-n\" \"ip\" \"link\" \"set\" \"{}\" \"down\"",
            "tap0"
        )
    );

    assert_eq!(
        format!("{:?}", result[1]),
        format!(
            "\"sudo\" \"-S\" \"-n\" \"ip\" \"tuntap\" \"del\" \"dev\" \"{}\" \"mode\" \"tap\"",
            "tap0"
        )
    );
}

pub fn exec_tap_cleanup(netdev_name: &NetdevName) -> Result<(), String> {
    let commands = gen_tap_cleanup(netdev_name);
    exec_sudo_commands(commands)
}
