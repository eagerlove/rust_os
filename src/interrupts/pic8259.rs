// Since interrupts 0x00 through 0x1F are reserved by the processor
// So we set PIC1 interrupts to 0x20-0x27
// set PCI2 interrupts to 0x28-0x2F

use x86_64::instructions::port::Port;

// command sent to begin PIC initialization
const CMD_INIT: u8 = 0x11;

// command sent to acknowledge an interrupt
const CMD_END_OF_INTERRUPT: u8 = 0x20;

// the mode in which we want to run ours PICs
const MODE_8086: u8 = 0x01;

// an individual PIC chip
struct Pic {
    offset: u8, // the base offset to which we send commands
    command: Port<u8>, // the processor I/O port on which we send commands
    data: Port<u8>, // the processro I/O port on which we send and receive data
}

impl Pic {
    fn handles_interrupt(&self, interupt_id: u8) -> bool {
        // each PIC handles 8 interrupts
        self.offset <= interupt_id && interupt_id < self.offset + 8
    }

    // notify us that an interrupt has been handled
    unsafe fn end_of_interrupt(&mut self) {
        self.command.write(CMD_END_OF_INTERRUPT);
    }

    // reads the interrupt mask of this PIC
    unsafe fn read_mask(&mut self) -> u8 {
        self.data.read()
    }

    // writes the interrupt mask of this PIC
    unsafe fn write_mask(&mut self, mask: u8) {
        self.data.write(mask)
    }
}

// a pair of chained PIC controllers
pub struct ChainedPics {
    pics: [Pic; 2],
}

impl ChainedPics {
    pub const unsafe fn new(offset1: u8, offset2: u8) -> ChainedPics {
        ChainedPics {
            pics: [
                Pic {
                    offset: offset1,
                    command: Port::new(0x20),
                    data: Port::new(0x21),
                },
                Pic {
                    offset: offset2,
                    command: Port::new(0xA0),
                    data: Port::new(0xA1),
                },
            ],
        }
    }

    pub unsafe fn initialize(&mut self) {
        let mut wait_port: Port<u8> = Port::new(0x80);
        let mut wait = || wait_port.write(0); // closure to block
        let saved_masks = self.read_masks();

        self.pics[0].command.write(CMD_INIT);
        wait();
        self.pics[1].command.write(CMD_INIT);


        self.pics[0].data.write(self.pics[0].offset);
        wait();
        self.pics[1].data.write(self.pics[1].offset);
        wait();

        self.pics[0].data.write(4);
        wait();
        self.pics[1].data.write(2);
        wait();

        self.pics[0].data.write(MODE_8086);
        wait();
        self.pics[1].data.write(MODE_8086);
        wait();

        self.write_masks(saved_masks[0], saved_masks[1])
     }

    pub unsafe fn read_masks(&mut self) -> [u8; 2] {
        [self.pics[0].read_mask(), self.pics[1].read_mask()]
    }

    pub unsafe fn write_masks(&mut self, mask1: u8, mask2: u8) {
        self.pics[0].write_mask(mask1);
        self.pics[1].write_mask(mask2);
    }

    pub unsafe fn disable(&mut self) {
        self.write_masks(u8::MAX, u8::MAX);
    }

    pub fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.pics.iter().any(|p| p.handles_interrupt(interrupt_id))
    }

    // figure out which PICs in our chain need to know about this interrupt
    pub unsafe fn notify_end_of_interrupt(&mut self, interrupt_id: u8) {
        if self.handles_interrupt(interrupt_id) {
            if self.pics[1].handles_interrupt(interrupt_id) {
                self.pics[1].end_of_interrupt();
            }
            self.pics[0].end_of_interrupt();
        }
    }

}