#![feature(question_mark)]
extern crate netdma;
extern crate drivers;
extern crate netbuf;

use std::env;
use std::mem;
use std::fmt;

use netdma::{DeviceList, PciDevice, Driver as DriverTrait, DmaMem};
use drivers::intel::e1000::Driver;
use netbuf::Ring;

#[derive(Copy, Clone)]
pub struct MacAddr {
    pub bytes: [u8; 6],
}

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct EthernetIIHeader {
    pub dst: MacAddr,
    pub src: MacAddr,
    pub ethertype: [u8; 2],
}

#[derive(Debug)]
pub struct EthernetII {
    pub header: EthernetIIHeader,
    pub data: Vec<u8>,
}



impl fmt::Debug for MacAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..6 {
            if i > 0 {
                write!(f,":")?;
            }
            write!(f, "{:X}", self.bytes[i])?;
        }
        Ok(())
    }
}

impl EthernetII {
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() >= mem::size_of::<EthernetIIHeader>() {
            unsafe {
                return Some(EthernetII {
                    header: *(bytes.as_ptr() as *const EthernetIIHeader),
                    data: bytes[mem::size_of::<EthernetIIHeader>()..].to_vec(),
                });
            }
        }
        None
    }
}

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

    // let cs = ConfigSpace::load(&d).unwrap();
    // println!("{:?}", cs);
    let (rx_ring, mut rx_controller) = Ring::<DmaMem>::new(1024,2048).unwrap();
    let (tx_ring, mut tx_controller) = Ring::<DmaMem>::new(1024,2048).unwrap();

    let mut drv = Driver::init(device, vec![rx_ring], vec![tx_ring]);

    // let status = drv.registers.status().read();
    // println!("STATUS: {:#b}", status);
    // println!("Full-duplex: {}", bool_bit(status, 0));
    // println!("Transmission Paused: {}", bool_bit(status, 4));
    // let speed = [10, 100, 1000, 1000][(0b11 & (status >> 6) as usize)];
    // println!("Speed: {} Mb/s", speed);
    //
    // println!("{:?}", drv.registers.status());






    loop {

        // println!("{:?}", drv.registers.status());
        // println!("{:?}", drv.registers.crcerrs());
        // println!("{:?}", drv.registers.algnerrc());
        // println!("{:?}", drv.registers.symerrs());
        // println!("{:?}", drv.registers.rxerrc());
        // println!("{:?}", drv.registers.mpc());
        // println!("{:?}", drv.registers.scc());
        // println!("{:?}", drv.registers.ecol());
        // println!("{:?}", drv.registers.mcc());
        // println!("{:?}", drv.registers.rlec());
        // println!("{:?}", drv.registers.gprc());
        // println!("{:?}", drv.registers.bprc());
        // println!("{:?}", drv.registers.mprc());
        // println!("{:?}", drv.registers.tpr());
        // println!("{:?}", drv.registers.rnbc());
        // println!("{:?}", drv.registers.ruc());
        // println!("{:?}", drv.registers.rfc());
        // println!("{:?}", drv.registers.roc());
        // println!("{:?}", drv.registers.rjc());
        // println!("-------------------------");

        for _ in 0..100_000 {
            drv.receive();

            while let Some(buf) = rx_controller.pop_queue() {
                let eth = EthernetII::from_bytes(buf.as_slice().to_vec());
                println!("received: {:?}", eth);
                println!("-------------------------");
            }
        }
    }

}
