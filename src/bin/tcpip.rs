#![feature(question_mark)]
extern crate netdma;
extern crate drivers;
extern crate netbuf;
extern crate tcpip;

use std::env;
use std::mem;
use std::fmt;

use netdma::{DeviceList, PciDevice, Driver as DriverTrait, DmaMem};
use drivers::intel::e1000::Driver;
use netbuf::Ring;
use tcpip::TcpIp;
use tcpip::ethernet::MacAddr;


fn main() {
    let dl = DeviceList::scan();
    println!("{:?}", dl);

    let addr = env::args().skip(1).next().unwrap();

    let dl = DeviceList::scan();
    let mut found = None;
    for d in dl.devices() {

        if d.pci_addr == addr {
            found = Some(d.pci_addr.clone());
            break;
        } else {
            if let Some(ref int) = d.interface {
                if int == &addr {
                    found = Some(d.pci_addr.clone());
                    break;
                }
            }
        }
    }

    if found.is_none() {
        return;
    }

    let d = found.unwrap();


    println!("taking over: {:?}", d);
    let device = PciDevice::unbind_and_aquire(d.clone()).unwrap();

    let (rx_ring, mut rx_controller) = Ring::<DmaMem>::new(1024, 2048).unwrap();
    let (tx_ring, mut tx_controller) = Ring::<DmaMem>::new(1024, 2048).unwrap();

    let mut drv = Driver::init(device, vec![rx_ring], vec![tx_ring]);

    let mut tcpip = TcpIp::new(MacAddr { bytes: drv.get_mac() },
                               rx_controller,
                               tx_controller)
                        .unwrap();
    loop {
        drv.receive();
        tcpip.process();
        drv.transmit();
    }
}
