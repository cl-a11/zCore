qemu-system-riscv64 -smp 1 -machine virt -bios default -m 512M -no-reboot -serial mon:stdio -serial file:/tmp/serial.out -kernel target/riscv64/release/zcore.bin -initrd zCore/riscv64.img -append "LOG=warn" -drive file=nvme.img,if=none,id=nvm -device nvme,serial=xxxxx,drive=nvm



qemu-system-riscv64 -smp 1 -machine virt -bios default -m 512M -no-reboot -serial mon:stdio -serial file:/tmp/serial.out -kernel target/riscv64/release/zcore.bin -initrd zCore/riscv64.img -append "LOG=warn" -drive file=nvme.img,if=none,id=nvm -device nvme,serial=xxxxx,drive=nvm
 -netdev user,id=net1,hostfwd=tcp::8000-:80,hostfwd=tcp::2222-:2222,hostfwd=udp::6969-:6969 -device e1000e,netdev=net1

 
qemu-system-riscv64 -M virt -bios /home/y/buildroot/output/images/fw_jump.elf -kernel /home/y/buildroot/output/images/Image -append "rootwait root=/dev/vda ro" -drive file=/home/y/buildroot/output/images/rootfs.ext2,format=raw,id=hd0 -device virtio-blk-device,drive=hd0 -nographic -drive file=/home/y/buildroot/nvme.img,if=none,id=nvm -device nvme,serial=xxxxx,drive=nvm

qemu-system-riscv64 -smp 1 -machine virt -bios default -m 512M -no-reboot -serial mon:stdio -serial file:/tmp/serial.out -kernel target/riscv64/release/zcore.bin -initrd zCore/riscv64.img -append "LOG=warn" -drive file=nvme.img,if=none,id=nvm -device nvme,serial=xxxxx,drive=nvm dumpdtb=./virt.dtb




qemu-system-riscv64 -smp 1 -machine virt -bios default -m 512M -drive file=nvme.img,if=none,id=nvm -device nvme,serial=xxxxx,drive=nvm -netdev tap,id=net1,script=ifup.sh,downscript=none -machine dumpdtb=./1.dtb





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
                        interrupt-map = <0x00 0x00 0x00 0x01 0x03 0x20 0x00 0x00 0x00 0x02 0x03 0x21 0x00 0x00 0x00 0x03 0x03 0x22 0x00 0x00 0x00 0x04 0x03 0x23 0x800 0x00 0x00 0x01 0x03 0x21 0x800 0x00 0x00 0x02 0x03 0x22 0x800 0x00 0x00 0x03 0x03 0x23 0x800 0x00 0x00 0x04 0x03 0x20 0x1000 0x00 0x00 0x01 0x03 0x22 0x1000 0x00 0x00 0x02 0x03 0x23 0x1000 0x00 0x00 0x03 0x03 0x20 0x1000 0x00 0x00 0x04 0x03 0x21 0x1800 0x00 0x00 0x01 0x03 0x23 0x1800 0x00 0x00 0x02 0x03 0x20 0x1800 0x00 0x00 0x03 0x03 0x21 0x1800 0x00 0x00 0x04 0x03 0x22>;
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




xp/64xg 0x30008000
0000000030008000: 0x0010000010d38086 0x0000000002000000
0000000030008010: 0x4006000040040000 0x4008000000000001
0000000030008020: 0x0000000000000000 0x0000808600000000
0000000030008030: 0x000000c800000000 0x0000010000000000
0000000030008040: 0x0000000000000000 0x0000000000000000
0000000030008050: 0x0000000000000000 0x0000000000000000
0000000030008060: 0x0000000000000000 0x0000000000000000
0000000030008070: 0x0000000000000000 0x0000000000000000
0000000030008080: 0x0000000000000000 0x0000000000000000
0000000030008090: 0x0000000000000000 0x0000000000000000
00000000300080a0: 0x0000000300040011 0x0000000000002003
00000000300080b0: 0x0000000000000000 0x0000000000000000
00000000300080c0: 0x0000000000000000 0x000000000022d001
00000000300080d0: 0x000000000080e005 0x0000000000000000
00000000300080e0: 0x000080000091a010 0x0000041100000000
00000000300080f0: 0x0000000000110000 0x0000000000000000
0000000030008100: 0x0000000014020001 0x0046203000000000
0000000030008110: 0x0000e00000000000 0x00000000000000a0
0000000030008120: 0x0000000000000000 0x0000000000000000
0000000030008130: 0x0000000000000000 0x0000000000000000
0000000030008140: 0xff12345600010003 0x00000000525400ff
0000000030008150: 0x0000000000000000 0x0000000000000000
0000000030008160: 0x0000000000000000 0x0000000000000000
0000000030008170: 0x0000000000000000 0x0000000000000000
0000000030008180: 0x0000000000000000 0x0000000000000000
0000000030008190: 0x0000000000000000 0x0000000000000000
00000000300081a0: 0x0000000000000000 0x0000000000000000
00000000300081b0: 0x0000000000000000 0x0000000000000000
00000000300081c0: 0x0000000000000000 0x0000000000000000
00000000300081d0: 0x0000000000000000 0x0000000000000000
00000000300081e0: 0x0000000000000000 0x0000000000000000
00000000300081f0: 0x0000000000000000 0x0000000000000000


xp/64xg 0x30008000
0000000030008000: 0x0010000010d38086 0x0000000002000000
0000000030008010: 0x4006000040040000 0x4008000000000001
0000000030008020: 0x0000000000000000 0x0000808600000000
0000000030008030: 0x000000c800000000 0x0000010000000000
0000000030008040: 0x0000000000000000 0x0000000000000000
0000000030008050: 0x0000000000000000 0x0000000000000000
0000000030008060: 0x0000000000000000 0x0000000000000000
0000000030008070: 0x0000000000000000 0x0000000000000000
0000000030008080: 0x0000000000000000 0x0000000000000000
0000000030008090: 0x0000000000000000 0x0000000000000000
00000000300080a0: 0x0000000300040011 0x0000000000002003
00000000300080b0: 0x0000000000000000 0x0000000000000000
00000000300080c0: 0x0000000000000000 0x000000000022d001
00000000300080d0: 0x000000000080e005 0x0000000000000000
00000000300080e0: 0x000080000091a010 0x0000041100000000
00000000300080f0: 0x0000000000110000 0x0000000000000000
0000000030008100: 0x0000000014020001 0x0046203000000000
0000000030008110: 0x0000e00000000000 0x00000000000000a0
0000000030008120: 0x0000000000000000 0x0000000000000000
0000000030008130: 0x0000000000000000 0x0000000000000000
0000000030008140: 0xff12345600010003 0x00000000525400ff
0000000030008150: 0x0000000000000000 0x0000000000000000



0000000030008000: 0x0018000700101b36 0x0000000001080202
0000000030008010: 0x0000000040000004 0x0000000000000000
0000000030008020: 0x0000000000000000 0x11001af400000000
0000000030008030: 0x0000004000000000 0x0000010000000000
0000000030008040: 0x0000200000408011 0x0000000000003000
0000000030008050: 0x0000000000000000 0x0000000000000000
0000000030008060: 0x0000000800030001 0x0000000000000000
0000000030008070: 0x0000000000000000 0x0000000000000000
0000000030008080: 0x1000800000926010 0x0000041100000000
0000000030008090: 0x0000000000110000 0x0000000000000000
00000000300080a0: 0x0030000000000000 0x0000000000000000
00000000300080b0: 0x0000000000000000 0x0000000000000000
