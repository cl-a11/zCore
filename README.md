qemu-system-riscv64 -smp 1 -machine virt,dumpdtb=./1.dtb -bios default -m 512M -no-reboot -serial mon:stdio -serial file:/tmp/serial.out -kernel target/riscv64/release/zcore.bin -initrd zCore/riscv64.img -append "LOG=warn" -drive file=./nvme.img,if=none,id=nvm -device nvme,serial=xxxxx,drive=nvm
dtc -I dtb -O dts ./1.dtb > ./1.dts


qemu-system-riscv64 -smp 1 -machine virt,dumpdtb=/home/y/zcore/1.dtb -bios default -m 512M -no-reboot -serial mon:stdio -serial file:/tmp/serial.out -kernel target/riscv64/release/zcore.bin -initrd zCore/riscv64.img -append "LOG=warn" -drive file=nvme.img,if=none,id=nvm -device nvme,serial=xxxxx,drive=nvm
 -netdev user,id=net1,hostfwd=tcp::8000-:80,hostfwd=tcp::2222-:2222,hostfwd=udp::6969-:6969 -device e1000e,netdev=net1

 
qemu-system-riscv64 -M virt -bios /home/y/buildroot/output/images/fw_jump.elf -kernel /home/y/buildroot/output/images/Image -append "rootwait root=/dev/vda ro" -drive file=/home/y/buildroot/output/images/rootfs.ext2,format=raw,id=hd0 -device virtio-blk-device,drive=hd0 -nographic -drive file=/home/y/buildroot/nvme.img,if=none,id=nvm -device nvme,serial=xxxxx,drive=nvm

qemu-system-riscv64 -smp 1 -machine virt -bios default -m 512M -no-reboot -serial mon:stdio -serial file:/tmp/serial.out -kernel target/riscv64/release/zcore.bin -initrd zCore/riscv64.img -append "LOG=warn" -drive file=nvme.img,if=none,id=nvm -device nvme,serial=xxxxx,drive=nvm dumpdtb=./virt.dtb




qemu-system-riscv64 -smp 1 -machine virt -bios default -m 512M -drive file=nvme.img,if=none,id=nvm -device nvme,serial=xxxxx,drive=nvm -machine dumpdtb=./1.dtb





(qemu) info pci
  Bus  0, device   0, function 0:
    Host bridge: PCI device 1b36:0008
      PCI subsystem 1af4:1100
      id ""
  Bus  0, device   1, function 0:
    Class 0264: PCI device 1b36:0010
      PCI subsystem 1af4:1100
      IRQ 15, pin A
      BAR0: 64 bit memory at 0x400000000 [0x400003fff].
      id ""




/dts-v1/;

/ {
        #address-cells = <0x02>;
        #size-cells = <0x02>;
        compatible = "riscv-virtio";
        model = "riscv-virtio,qemu";

        fw-cfg@10100000 {
                dma-coherent;
                reg = <0x00 0x10100000 0x00 0x18>;
                compatible = "qemu,fw-cfg-mmio";
        };

        flash@20000000 {
                bank-width = <0x04>;
                reg = <0x00 0x20000000 0x00 0x2000000 0x00 0x22000000 0x00 0x2000000>;
                compatible = "cfi-flash";
        };

        chosen {
                bootargs = [00];
                stdout-path = "/soc/uart@10000000";
        };

        memory@80000000 {
                device_type = "memory";
                reg = <0x00 0x80000000 0x00 0x20000000>;
        };

        cpus {
                #address-cells = <0x01>;
                #size-cells = <0x00>;
                timebase-frequency = <0x989680>;

                cpu@0 {
                        phandle = <0x01>;
                        device_type = "cpu";
                        reg = <0x00>;
                        status = "okay";
                        compatible = "riscv";
                        riscv,isa = "rv64imafdcsuh";
                        mmu-type = "riscv,sv48";

                        interrupt-controller {
                                #interrupt-cells = <0x01>;
                                interrupt-controller;
                                compatible = "riscv,cpu-intc";
                                phandle = <0x02>;
                        };
                };

                cpu-map {

                        cluster0 {

                                core0 {
                                        cpu = <0x01>;
                                };
                        };
                };
        };

        soc {
                #address-cells = <0x02>;
                #size-cells = <0x02>;
                compatible = "simple-bus";
                ranges;

                rtc@101000 {
                        interrupts = <0x0b>;
                        interrupt-parent = <0x03>;
                        reg = <0x00 0x101000 0x00 0x1000>;
                        compatible = "google,goldfish-rtc";
                };

                uart@10000000 {
                        interrupts = <0x0a>;
                        interrupt-parent = <0x03>;
                        clock-frequency = <0x384000>;
                        reg = <0x00 0x10000000 0x00 0x100>;
                        compatible = "ns16550a";
                };

                poweroff {
                        value = <0x5555>;
                        offset = <0x00>;
                        regmap = <0x04>;
                        compatible = "syscon-poweroff";
                };

                reboot {
                        value = <0x7777>;
                        offset = <0x00>;
                        regmap = <0x04>;
                        compatible = "syscon-reboot";
                };

                test@100000 {
                        phandle = <0x04>;
                        reg = <0x00 0x100000 0x00 0x1000>;
                        compatible = "sifive,test1\0sifive,test0\0syscon";
                };

                pci@30000000 {
                        interrupt-map-mask = <0x1800 0x00 0x00 0x07>;
                        interrupt-map = <0x00 0x00 0x00 0x01 0x03 0x20 
                        0x00 0x00 0x00 0x02 0x03 0x21 
                        0x00 0x00 0x00 0x03 0x03 0x22 
                        0x00 0x00 0x00 0x04 0x03 0x23 
                        0x800 0x00 0x00 0x01 0x03 0x21 
                        0x800 0x00 0x00 0x02 0x03 0x22 
                        0x800 0x00 0x00 0x03 0x03 0x23 
                        0x800 0x00 0x00 0x04 0x03 0x20 
                        0x1000 0x00 0x00 0x01 0x03 0x22 
                        0x1000 0x00 0x00 0x02 0x03 0x23 
                        0x1000 0x00 0x00 0x03 0x03 0x20 
                        0x1000 0x00 0x00 0x04 0x03 0x21 
                        0x1800 0x00 0x00 0x01 0x03 0x23 
                        0x1800 0x00 0x00 0x02 0x030x03 0x21 
                        0x1800 0x00 0x00 0x04 0x03 0x22>;
                        ranges = <0x1000000 0x00 0x00 0x00 0x3000000 0x00 0x10000 0x2000000 0x00 0x40000000 0x00 0x40000000 0x00 0x40000000 0x3000000 0x04 0x00 0x04 0x00 0x04 0x00>;
                        reg = <0x00 0x30000000 0x00 0x10000000>;
                        dma-coherent;
                        bus-range = <0x00 0xff>;
                        linux,pci-domain = <0x00>;
                        device_type = "pci";
                        compatible = "pci-host-ecam-generic";
                        #size-cells = <0x02>;
                        #interrupt-cells = <0x01>;
                        #address-cells = <0x03>;
                };

                virtio_mmio@10008000 {
                        interrupts = <0x08>;
                        interrupt-parent = <0x03>;
                        reg = <0x00 0x10008000 0x00 0x1000>;
                        compatible = "virtio,mmio";
                };

                virtio_mmio@10007000 {
                        interrupts = <0x07>;
                        interrupt-parent = <0x03>;
                        reg = <0x00 0x10007000 0x00 0x1000>;
                        compatible = "virtio,mmio";
                };

                virtio_mmio@10006000 {
                        interrupts = <0x06>;
                        interrupt-parent = <0x03>;
                        reg = <0x00 0x10006000 0x00 0x1000>;
                        compatible = "virtio,mmio";
                };

                virtio_mmio@10005000 {
                        interrupts = <0x05>;
                        interrupt-parent = <0x03>;
                        reg = <0x00 0x10005000 0x00 0x1000>;
                        compatible = "virtio,mmio";
                };

                virtio_mmio@10004000 {
                        interrupts = <0x04>;
                        interrupt-parent = <0x03>;
                        reg = <0x00 0x10004000 0x00 0x1000>;
                        compatible = "virtio,mmio";
                };

                virtio_mmio@10003000 {
                        interrupts = <0x03>;
                        interrupt-parent = <0x03>;
                        reg = <0x00 0x10003000 0x00 0x1000>;
                        compatible = "virtio,mmio";
                };

                virtio_mmio@10002000 {
                        interrupts = <0x02>;
                        interrupt-parent = <0x03>;
                        reg = <0x00 0x10002000 0x00 0x1000>;
                        compatible = "virtio,mmio";
                };

                virtio_mmio@10001000 {
                        interrupts = <0x01>;
                        interrupt-parent = <0x03>;
                        reg = <0x00 0x10001000 0x00 0x1000>;
                        compatible = "virtio,mmio";
                };

                plic@c000000 {
                        phandle = <0x03>;
                        riscv,ndev = <0x35>;
                        reg = <0x00 0xc000000 0x00 0x600000>;
                        interrupts-extended = <0x02 0x0b 0x02 0x09>;
                        interrupt-controller;
                        compatible = "sifive,plic-1.0.0\0riscv,plic0";
                        #interrupt-cells = <0x01>;
                };

                clint@2000000 {
                        interrupts-extended = <0x02 0x03 0x02 0x07>;
                        reg = <0x00 0x2000000 0x00 0x10000>;
                        compatible = "sifive,clint0\0riscv,clint0";
                };
        };
};



0xffffffff6fe00000..0xffffffff7fe00000 => 0x30000000