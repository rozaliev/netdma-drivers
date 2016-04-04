use PciDevice;
use netbuf::{Ring, Allocator};
use libc::c_void;
use netdma::{self, DmaMem};



registers! {
    E1000 {
        // General
        ctrl: 0x0 u32 RW bits {
            FD 1,
            LRST 3,
            ASDE 5,
            SLU 6,
            ASDE_SLU [5,6],
            ILOS 7,
            PHY_RST 31

        },
        status: 0x8 u32 R bits {
            FD 0,
            LU 1,
            TXOFF 4,
            TBIMODE 5,
            SPEED [6,7],
        },
        eecd: 0x10 u32 R,
        fcal: 0x28 u32 RW,
        fcah: 0x2C u32 RW,
        fct: 0x30 u32 RW,

        ims: 0xD0 u32 RW,


        rctl: 0x100 u32 RW bits {
            EN 1,
            SBP 2,
            UPE 3,
            MPE 4,
            LPE 5,
            LBM [6,7],
            BAM 15,
            BSIZE0 16,
            BSIZE1 17,
            BSEX 25,
            SECRC 26

        },
        fcttv: 0x170 u32 RW,

        rdbal: 0x2800 u32 RW,
        rdbah: 0x2804 u32 RW,
        rdlen: 0x2808 u32 RW,
        rdh: 0x2810 u32 RW,
        rdt: 0x2818 u32 RW,


        ral0: 0x5400 u32 RW,
        rah0: 0x5404 u32 RW,




        // Statistics
        crcerrs: 0x4000 u32 R,
        algnerrc: 0x4004 u32 R,
        symerrs: 0x4008 u32 R,
        rxerrc: 0x400C u32 R,
        mpc: 0x4010 u32 R,
        scc: 0x4014 u32 R,
        ecol: 0x4018 u32 R,
        mcc: 0x401C u32 R,
        //...
        rlec: 0x4040 u32 R,
        //...
        gprc: 0x4074 u32 R,
        bprc: 0x4078 u32 R,
        mprc: 0x407C u32 R,
        //...
        rnbc: 0x40A0 u32 R,
        ruc: 0x40A4 u32 R,
        rfc: 0x40a8 u32 R,
        roc: 0x40AC u32 R,
        rjc: 0x40b0 u32 R,

        //..
        tpr: 0x40D0 u32 R,





     }
}


pub struct Driver<A: Allocator> {
    rx_ring: Ring<A>,
    tx_ring: Ring<A>,
    device: PciDevice,
    pub registers: E1000,
}

impl<A: Allocator> netdma::Driver<A> for Driver<A> {
    fn init(device: PciDevice, mut rx: Vec<Ring<A>>, mut tx: Vec<Ring<A>>) -> Driver<A> {
        Driver::new(device, rx, tx)
    }

    fn num_of_queues() -> usize {
        1
    }

    fn receive(&mut self) {
        self.do_receive()
    }

    fn transmit(&mut self) {}
}

impl<A: Allocator> Driver<A> {
    fn new(device: PciDevice, mut rx: Vec<Ring<A>>, mut tx: Vec<Ring<A>>) -> Driver<A> {
        if rx.len() != 1 || tx.len() != 1 {
            panic!("this devices supports only 1 queue");
        }

        let rx = rx.pop().unwrap();
        let tx = tx.pop().unwrap();

        // restart card, otherwise old data might be deep in register

        let mut regs = unsafe { E1000::new(device.mem_ptr()) };

        regs.ctrl.enable_ASDE_SLU();

        regs.ctrl.disable_LRST();
        regs.ctrl.disable_PHY_RST();
        regs.ctrl.disable_ILOS();

        regs.fcah.write(0);
        regs.fcal.write(0);
        regs.fct.write(0);
        regs.fcttv.write(0);


        regs.rctl.disable_EN();


        let ring_phy_addr = rx.nic_ring_phy_addr();


        let ring_hi = ring_phy_addr >> 32;
        let ring_lo = ring_phy_addr & 0xffffffff;


        regs.rdbah.write(ring_hi as u32);
        regs.rdbal.write(ring_lo as u32);
        regs.rdlen.write((rx.nic_ring_size() * 16) as u32);
        regs.rdh.write(0);
        regs.rdt.write(rx.nic_ring_size() as u32 - 1);

        regs.rctl.enable_EN();
        regs.rctl.enable_UPE();
        regs.rctl.enable_LPE();
        regs.rctl.disable_LBM();
        regs.rctl.enable_BAM();
        // fix sizes
        regs.rctl.enable_BSIZE0();
        regs.rctl.disable_BSIZE1();
        regs.rctl.enable_BSEX();
        regs.rctl.enable_SECRC();

        Driver {
            rx_ring: rx,
            tx_ring: tx,
            registers: regs,
            device: device,
        }

    }


    fn do_receive(&mut self) {
        let ring_size = self.rx_ring.nic_ring_size();

        while self.registers.rdh.read() != self.registers.rdt.read() &&
              self.rx_ring.head_is_ready() {
            self.rx_ring.advance(1);
            let mut rdt = self.registers.rdt.read();
            rdt = (rdt + 1) % ring_size as u32;
            self.registers.rdt.write(rdt);
        }
    }
}

impl<A: Allocator> Drop for Driver<A> {
    fn drop(&mut self) {
        self.registers.rctl.disable_EN();
    }
}

const RD_DD: u8 = 1;
const RD_EOP: u8 = 1 << 1;
