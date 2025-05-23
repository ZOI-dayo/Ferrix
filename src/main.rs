use std::process::Command;
// use std::io::{BufRead, BufReader, Write};
// use std::net::{TcpListener, TcpStream};
use tun_tap::Iface;

// const TUNSETIFF: c_int = 0x400454CA;
// const IFF_TUN: c_short = 0x0001;
// const IFF_NO_PI: c_short = 0x1000;

fn main() {
    println!("Hello, world!");
    let iface = Iface::new("tap0", tun_tap::Mode::Tap).expect("failed to open tun file");

    let name = iface.name();
    println!(
        "{}",
        String::from_utf8(
            Command::new("sh")
                .arg("-c")
                .arg(format!("ip a add 192.168.178.2/24 dev {}", name))
                .output()
                .expect("failed to execute process")
                .stdout
        )
        .unwrap()
    );
    println!(
        "{}",
        String::from_utf8(
            Command::new("sh")
                .arg("-c")
                .arg(format!("ip link set {} up", name))
                .output()
                .expect("failed to execute process")
                .stdout
        )
        .unwrap()
    );

    println!("{:?}", iface);
    loop {
        let mut data = vec![0; 1504];
        let len = iface.recv(&mut data).expect("failed to receive data");
        println!("---");
        // println!("{}, {:x?}", len, &data[0..len]);
        println!("begin: {:x?}", &data[0..4]);
        println!("dest mac: {:x?}", &data[4..10]);
        println!("src mac: {:x?}", &data[10..16]);
        println!("EtherType: {:x?}", &data[16..18]);
        println!("Payload: {:x?}", &data[18..len - 4]);
        // decode ipv6 packet
        {
            println!("  version: {:x?}", &data[18] | 0b11110000);
            // println!(
            //     "  trafic class: {:x?}",
            //     (&data[19] | 0b00001111) as i64 * 256 + (&data[20] | 0b11110000) as i64
            // );
            // println!("  flow label")
            println!("  payload len: {:x?}", &data[22..24]);
            println!("  nxt header: {:x?}", &data[24]);
            println!("  hop limit: {:x?}", &data[25]);
            println!("  src addr: {:x?}", &data[26..42]);
            println!("  dst addr: {:x?}", &data[42..58]);
            println!("  content: {:x?}", &data[58..len])
            // todo: decode ndp packet
        }
    }

    // let tun_path = Path::new("/dev/net/tun");
    // unsafe {
    //     fcntl(
    //         tun_path.as_os_str().as_bytes().as_ptr() as i32,
    //         TUNSETIFF,
    //         libc::ifreq {
    //             ifr_name: [
    //                 b'e' as c_char,
    //                 b't' as c_char,
    //                 b'h' as c_char,
    //                 b'0' as c_char,
    //                 0,
    //                 0,
    //                 0,
    //                 0,
    //                 0,
    //                 0,
    //                 0,
    //                 0,
    //                 0,
    //                 0,
    //                 0,
    //                 0,
    //             ],
    //             ifr_ifru: __c_anonymous_ifr_ifru {
    //                 ifru_flags: IFF_NO_PI | IFF_TUN,
    //             },
    //         },
    //     );
    // }
    // let mut file = match File::open(&tun_path) {
    //     Err(why) => panic!("couldn't open {}: {}", tun_path.display(), why),
    //     Ok(file) => file,
    // };
    // println!(
    //     "{}",
    //     String::from_utf8(
    //         Command::new("sh")
    //             .arg("-c")
    //             .arg("ip a")
    //             .output()
    //             .expect("failed to execute process")
    //             .stdout
    //     )
    //     .unwrap()
    // );
    //
    // println!(
    //     "{}",
    //     String::from_utf8(
    //         Command::new("sh")
    //             .arg("-c")
    //             .arg("ip tuntap add dev tun0 mode tun")
    //             .output()
    //             .expect("failed to execute process")
    //             .stdout
    //     )
    //         .unwrap()
    // );
    // println!(
    //     "{:?}",
    //     Command::new("sh")
    //         .arg("-c")
    //         .arg("ip link set tun0 up")
    //         .output()
    //         .expect("failed to execute process")
    // );
    // println!(
    //     "{:?}",
    //     Command::new("sh")
    //         .arg("-c")
    //         .arg("ip addr add 192.168.0.1/24 dev tun0")
    //         .output()
    //         .expect("failed to execute process")
    // );
    // println!(
    //     "{}",
    //     String::from_utf8(
    //         Command::new("sh")
    //             .arg("-c")
    //             .arg("ip a")
    //             .output()
    //             .expect("failed to execute process")
    //             .stdout
    //     )
    //     .unwrap()
    // );
    // let mut data: Vec<u8> = vec![0; 1500];
    // loop {
    //     let tun = file
    //         .read_to_end(&mut data)
    //         .expect("failed to read tun data");
    //     println!("Read {} bytes from tun0", tun);
    //     println!("Content: {:?}", String::from_utf8_lossy(&data));
    // }

    // let listener = TcpListener::bind("0.0.0.0:80").unwrap();
    // for stream in listener.incoming() {
    //     let stream = stream.unwrap();
    //     handle_connection(stream);
    // }
}

// fn handle_connection(mut stream: TcpStream) {
//     let mut packet = BufReader::new(&stream);
//     let mut buffer: Vec<u8> = Vec::new();
//     loop {
//         let mut temp_buffer: Vec<u8> = Vec::new();
//         packet.read_until(b'\n', &mut temp_buffer).unwrap();
//         if temp_buffer.is_empty() {
//             // No more data to read
//             break;
//         }
//         buffer.extend_from_slice(&temp_buffer);
//         if temp_buffer.len() == 2 && temp_buffer == b"\r\n" {
//             // End of headers
//             // TODO: read content
//             break;
//         }
//         println!("Request: {:?}", temp_buffer.clone());
//         println!("Request: {}", String::from_utf8(buffer.clone()).unwrap());
//     }
//     println!("Request: {}", String::from_utf8(buffer).unwrap());
//     let response_body = "Hello, world!";
//     let response = format!(
//         "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
//         response_body.len(),
//         response_body
//     );
//     stream.write_all(response.as_bytes()).unwrap();
//     stream.flush().unwrap();
// }
