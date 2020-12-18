use hex_literal::hex;
use rusb::{Context, Device, DeviceHandle, Result, UsbContext};
use std::time::Duration;

// device uid pid are picked directly form `lsusb` result
const VID: u16 = 0x04b4;
const PID: u16 = 0x8613;

pub fn init() -> Result<(Device<impl UsbContext>, DeviceHandle<impl UsbContext>)>
{
    let mut context = Context::new()?;
    let (_, mut handle) =
        open_device(&mut context, VID, PID).expect("Failed to open USB device");

    handle.reset()?;

    initialize_device(&mut handle)?;

    // We reopen because the old handle doesn't reflect reality. libusb bug? Usage issue?
    // Probably just a general misunderstanding about how long handles are valid.
    let (device, handle) =
        open_device(&mut context, VID, PID).expect("Failed to open USB device");

    Ok((device, handle))
}

fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<(Device<T>, DeviceHandle<T>)> {
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => return Some((device, handle)),
                Err(_) => continue,
            }
        }
    }

    None
}

// This is all from recordings and a blackbox
fn initialize_device<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
) -> Result<()> {
    handle
        .set_active_configuration(1)
        .expect("Failed to set active configuration");
    handle
        .claim_interface(0)
        .expect("Claiming interface failed");
    handle
        .set_alternate_setting(0, 0)
        .expect("Failed to set the interface");

    let one_second = Duration::from_secs(1);

    for (val, data) in [
        (0xe600, &[1][..]),
        (0x1100, &hex!("1201000200000040b40413860100010200010a06000200000040010009023c00010100a0320904000006ff00000007050102400000070502020002000705040200020007058102400000070586020002000705880200020009023c0001010080320904000006ff000000070501024000000705020240000007050402400000070581024000000705860240000007058802400000040309040e0347005100200055005300420024034500500052004f004d002000500072006f006700720061006d006d006500720020000000")[..]),
        (0x19bc, &hex!("00010202030304040505")[..]),
        (0x15c5, &hex!("ec4efeed4f24bcf58274193ef583e493ff3395e0feef24a1ffee34e68f82f5832290e6bce0547eff7e00e0d394807c0022f0e5242401f524e43523f523e43522f522e43521f52122af28ae27ad26ac25ab24aa23a922a821c3021258e52e2524f582e52d3523f58322")[..]),
        (0x0fa1, &hex!("e4f52cf52bf52af529c203c200c202c20112134812162e12199b7e117f008e0a8f0b75121175131275081175091c751011751158751411751594ee54e07003021093752d00752e808e2f8f30c374ca9fff74119ecf2402cf3400fee48f288e27f526f525f524f523f522f52112160d500a12162174cd1215f680f1e4f524f523f522f52112160d5017e5302524f582e52f3523f583e0ff121621ef1215f680e4852d0a852e0b74002480ff741134fffec3e5139ff513e5129ef512c3e50d9ff50de50c9ef50cc3e50f9ff50fe50e9ef50ec3e5099ff509e5089ef508c3e5119ff511e5109ef510c3e5159ff515e5149ef514d2e843d82090e668e04409f090e65ce0443df0d2af90e680e020e105d2041210d090e680e054f7f0538ef8c203300105120df7c20112191512162e80f0")[..]),
        (0x0df7, &hex!("90e6b9e07003020ea8147003020efc24fe7003020f5624fb7003020ea2147003020e9c14607314607624056003020f8d1200514003020f9990e6bbe024fe602214603324fd601114602224067048e50a90e6b3f0e50b8037e51290e6b3f0e513802de50c90e6b3f0e50d8023e50e90e6b3f0e50f801990e6bae0ff121738aa06a9077b01ea494b600dee90e6b3f0ef90e6b4f0020f99020f921219eb020f991219d6020f991219ce020f991219e5020f991219f54003020f9990e6b8e0247f60151460192402703aa200e43325e0ffa202e4334f8018e490e740f080161215e640047d0180027d001215c5e0540190e740f0e4a3f090e68af090e68b7402f0020f99020f921219f74003020f9990e6b8e024fe601624026003020f9990e6bae0b40105c200020f99020f9290e6bae0702c1215e640047d0180027d001215c5e054fef090e6bce05480131313541fffe0540f2f90e683f0e04420f08045803c1219f9503e90e6b8e024fe601a2402703290e6bae0b40104d200802790e6bae0b40202801e80151215e640047d0180027d001215c580081219fb500790e6a0e04401f090e6a0e04480f022")[..]),
        (0x0033, &hex!("0219f1")[..]),
        (0x19f1, &hex!("53d8ef32")[..]),
        (0x0043, &hex!("021400")[..]),
        (0x0053, &hex!("021400")[..]),
        (0x1400, &hex!("0218eb0002194f0002193c000219000002178f0002170600020032000210ff000213ff000219fd0002196200021975000218bc000219fe00021988000219ff00021a00000210ff00021a0100021a0200021a0300021a0400021a0500021a0600021a07000210ff000210ff000210ff00021a0800021a0900021a0a00021a0b00021a0c00021a0d00021a0e00021a0f00021a1000021a1100021a1200021a1300021a1400021a1500021a1600021a1700021a1800021a1900")[..]),
        (0x1846, &hex!("014e00014f00014a00035101e7c0015401015000014900015500014b00024c0000")[..]),
        (0x128f, &hex!("0531af31053174562ff8e6221219aeab51aa52a953ae5505558e82758300227f0a7e001218aa85803bab51aa52a953af5505558f8275830022053174562531f8e622e53212122474072532f582e4343cf58322ac39ad3aaf547e00021246ab3f2541f9e5403efa1211ccffe5445407fe7401a8060822e4fbfdff02176eef1212240532227f887e130218aa74872532f582e4343cf58322f582e434e7f583e0ff22ae31053174562ef8e622ab3caa3da93e22e544ae43780322")[..]),
        (0x18a1, &hex!("af547c077dd0121246e4fdfcc3ed9fec9e50070dbd00010c80f222")[..]),
        (0x1824, &hex!("7fc87e00ab07aa06d28ce4f54cf54dad03ac021212e6c3e54d9fe54c9e40f0c28c22")[..]),
        (0x19de, &hex!("eff4f5b28f8022")[..]),
        (0x1764, &hex!("7858e6ff08e6fd08e6fbd2a1d2a18bb1c2a5d2a5c2a58db1c2a6d2a6c2a68fb1c2a7d2a7c2a7c2a1c2a122")[..]),
        (0x0036, &hex!("121305d2a3d2a21213130218aa")[..]),
        (0x0046, &hex!("c2a4c2a2e4fbfdff02176e")[..]),
        (0x19ae, &hex!("e4f5b2c2a07f0afe1218aaaf8022")[..]),
        // Packet 77
        (0x1544, &hex!("ae03e54a602975b3fbefd394004004d2938002c293edd394004004d2958002c295eb6005a292e433fbd294c2948024efd394004004d2b28002c2b2edd394004004d2b38002c2b3eb6005a2b0e433fbd2b1c2b1eed394004025ebd394004010e548b401081212b874011212247f0122e548b401071212b8e41212247f0022af0622")[..]),
        (0x1885, &hex!("e4f54500000090e68feff090e6a2e020e1f97551017552e77553c022")[..]),
        (0x18d5, &hex!("8f457f3c120003af451200037f3e1200037f3e020003")[..]),
        (0x1868, &hex!("8f458d467f3c120003af45120003af461200037f3e1200037f3e020003")[..]),
        (0x000e, &hex!("8f458d468b477f3c120003af45120003af46120003af471200037f3e1200037f3e020003")[..]),
        (0x17de, &hex!("e55414600a14600e14600b2403700e90e6007410f02290e6007408f022e490e600f022")[..]),
        (0x0056, &hex!("e4f535f536f555f53190e68de0ffe531c39f50127480253112132674562531f8a607053180e390e68de0f534e4f0903c00e05410d394005008a3e0542494004005754f018003e4f54fe4f531e531c395344003020b481212ca12126901fb02033303041704047a05041706047a07033f0803490903ae0a03fe0b040a0c06d20d05a00e05b30f05c81005eb1105fe12061113062114063415063c16064e1702301806441906581a06671b01801c019c1d06874201ba46016a4703004808d456024d6c02b66e026c6f02c77001f27301f47501e57701a378013d7a02e97b029c7c02747d00000b3d7f3c800be531c395345009121291ff12000380f0120d61e4f531120cbcef1212240531e531c3941a40f0020b4005317f471218d5120cbc120bd3120bd3ef121224801212128ff532121291f533fbad327f1c12000e1219c612129eef020b38120d6c7f1d800c120d6c7f78120003121291ff120c16120d3f0531020b4005317857e6f5377f461218d5800aaf371537efd394004010121291ff120003121291ff12000380e602042005317f771218d51219c6020628800005310531020b4012128f7f02120ddf1219c68f331212b8e5331212247e0074ff2555f582ee34fff5831211e560090532e532c3942040d80531020b4012128f7f18120ddf1219c612129e12130ce532c3940b40f00531020b4012128ffd7f6c121868e4f5321219c612129e12130ce532c3940740f0020b40120dc57f6f0202a90531e4120de77f7d120b9ff537120d3fe4f532e532c39537500b1219c612129e12130c80ee0202e1120dc57f7c120b5eff120c16ff120c3d120c64ef1212240202e1120def7f6e120b9f120c64ef121224801a120def7f70120b5eff1200037f3e1200037f3e120ca6ef121224e53404f531020b4005317f3c1200037f7b120c3dff120003121291ff02084012128f120de77f48120003af37120003120d617ff47e011218aae4f532e532c395374003020b40121291ff120ca612130c80ea1212b8e41212240531020b4012128ff5490531020b400531c2a2c2a3c2a01213051219de120d747f05120d2c121824d2a3d2a0d2a27e001218a1d2a4e4fb7d02ff12176efe1218a1c2a3c2a4af547e007c137d881212461218aad2a47ff47e01121828d2a3c2a4c2a2c2a4121305120ccf1218241212b8e40204050531120d747f01120d2c121824e4fb7d02ff12176ed2a2d2a4c2a3c2a012176e7fc8fe12182812129bef1212247b017d02e4ff12176e12131312129bef121224d2a3d2a0c2a2c2a4120ccf121824802205311212b87401121224801605311212b87401121224020b3d05311217dee5496003020b3d121764c2a3c2a0785de6700408e6603112176412129bef121224e555c394404003120d93785a06e670071806e670021806785e16e670d018e6d3940040c91680c6e555d394004003120d931212b8e412122474090205e4053174042531f531e5496003020b3d753801753900753a01753900753a0ad2a3121305e531c39534400302058e121764121291f537f47003020575d2a3d2a0af371219ded2a41212e2ef7802c333ce33ced8f9ff1218aac2a31212e2ef7802c333ce33ced8f9ff1218aad2a3c2a4c2a3c2a01212e2ef7802c333ce33ced8f9ff1218aae4ff1219de1219aeef65376065e5386414704fab51aa52a953af5505558f82f583041212247857e61212a41212247858e61212a41212247859e61212a4121224785ae61212a41212241212bee53712122412129bef121224c2a4853431754901802f7414253af53ae43539f53905380204b5753900753a14753801785a06e6600302049d1806e6600302049d180602049dc2a4e555d394005003020b40120dcd020b401212c8d394004004d2a48002c2a40531020b401212c8d394004006d2a3d2a28002c2a20531020b40053174572531f8e6ff74582531f8e6fd74592531f8e6fb12176e74052531f531020b401212c8d394004004d2a08002c2a00531020b401212c8d394004004d2a38002c2a30531020b400531d2a01212caff1219de0531020b400531c2a01219ae12129eef1212240531020b400531120046020b400531120036020b401212c8f5540531020b401212c8f5500531020b4012128f903c05f0121291a3f0020b400531903c05e012129e121224903c06e01212a4121224d2a4121313c2a4020b4012128ff5377f3c1200037f42120003af37120003e4f532e532c39537501c1212caff74002532f582e4343cf583eff0121291ff120003053280dd120da5d2957f407e9c1218aa0531020b4012128ff537646070237f3c1200037f521200037f451200037f421200037f4f1200037f4f1200037f54020840e53764886003020819121291f538121291f539121291f53ae4f5b2c2a2c2a4e5387004c2938002d293e5397004c2958002d295e53a7004c2948002d294e53864017019e5397015e53a7011f53bf5b2121291f5b11212aee53b020b38e538640160030207e8e53960030207e8e53a64017078f53bf5b2121291f5375401d394004003d38001c392a0e5375402d394004003d38001c392a1e5375404d394004003d38001c39292e5375408d394004003d38001c392a3e5375420d394004003d38001c392a5e5375440d394004003d38001c392a6e5375480d394004003d38001c392a71212aee53b020b38e538700de5397009e53a7005f5b2020b40e53864016003020b40e53964016003020b40e53a64016003020b40f5b2020b40e5376489700f754a01120db07f141200037f0a8012e537648a7015f54a120db07f1e1200037f0a120003120da5020b40e4f5321212d67455f00532e532b480f3e4f53212131a74aaf00532e532b480f3e4f5321212d6e0645560171212b874551212241212be1212d1e01212a412122480070532e532b480dae4f53212131ae064aa60111212b874aa1212241212be1212d1e0801e0532e532b480e0e531c395344003020b4090e60ae012129e1212241212911212a412122480e112128f146028146054147003020ad3147003020b0824046003020b40121291ff121330fd121330fb754801020b0312129114601514601c24026003020b40e4f548fbfd120d4e020b03120c85120d9cff020b03120c85120d9cff020b03e4f537f538f539f53af53b753c01753d3c753e07753f0175403c754107f542f543f544121291753700f538753800f5371212914238121291f53a121291f53b90e68de0d3940040f7e4f543f54490e68de0ffc3e5449fe5439400502674802544121326e5427c002544fdec354312133a8d82f583ef1212240544e54470cf054380cbe5442542f542e490e68df0e5382407ffe43537feef7803cec313ce13d8f9ffc3e5429fe49e4096e4f543f544c3e5449538e54395374003020a92e53824ffffe53734fffeefb54408eeb543047f0180027f00c007121341cec313ce13d8f912133af5828e831211e5fd7c00e5445407ff74017e00a807088005c333ce33ced8f9ffee5cfeef5d4e60047d0180027d00e4f548d007ab3b1215448f39e53b64017033e539b40116121341cec313ce13d8f91212ed8002c333d8fc4f8015121341cec313ce13d8f91212ed8002c333d8fcf45f1212120544e544700205430209dfe4f555f543f544c3e5449542e5439400502612133a8544828543831211e512129e121224e555c394404003120dcd0544e54470d3054380cfe4f548fbfdff020b0312129114601a14602024027060e4f548fbfd7f01121544e4f548fb120d4e8010120d02120dd6ff8007120d02120dd6ff1215448038785ae6700a18e6700618e67002801d785be6ffe4f548fbfd121544785a16e6b4ffde1816e6b4ffd8181680d41212b8e41212248003853431e53404f5310200a2e555d394004008af55121885e4f55590e6007410f022120003af31053174562ff8e6ff120003af31053174562ff8e6ff120003af31053174562ff8e6ff120003af31053174562ff8e6ff120003af31053174562ff8e622120003af31053174562ff8e6ff120003af31053174562ff8e6ff120003af31053174562ff8e6ff120003af31053174562ff8e622ef1212241219c6ab51aa52a953ae5505558e82758300ef1212241219c6ab51aa52a953ae5505558e82758300ef1212241219c6ab51aa52a953ae5505558e8275830022120003af31053174562ff8e6ff120003af31053174562ff8e6ff120003af31053174562ff8e622120003af31053174562ff8e6ff120003af31053174562ff8e6ff120003af31053174562ff8e622ff1200037f3e1200037f3e1200031219c6ab51aa52a953ae5505558e8275830022e4f548fbfd7f01121544e4f548fb7f01121544e4f548fb7f01121544e4f548fb221200031219c6ab51aa52a953ae5505558e82758300221219c6ab51aa52a953ae5505558e82758300227f3c1200037f421200037f051200037f8f1200037f4a1200037f041200037f041200037f6f1200037f3e1200037f3e12000322e4f548fbfd7f01121544e4f548fb7f01121544e4f548fb7f01121544e4f548fb7f01121544e4f548fb221200037f6f1200037f3e1200037f3e12000322ff1200037f3e1200037f3e120003227f01121544e4f548fbff121544e4f548fbff227f3e1200037f3e1200032205317f3c120003227f3c1200037f421200037f051200037f8f1200037f4a1200037f0412000322af55121885e4f55522ff121544e4f548fb227f3e1200037f3e120003227f3c1200037f7b1200037f011200037f021200032205317f3c12000322af55121885e4f55522ff121544e4f548fb22fd121868e4f53222f5377f3c1200032205317f3c12000322")[..]),
        (0x1801, &hex!("c2a9e58954f04401f589438e08c28d758cd1758a20e4f54cf54dd2afd2a9d2b9c28c22")[..]),
        (0x000b, &hex!("021929")[..]),
        (0x1929, &hex!("c0e0758cd1758a20054de54d7002054cd0e032")[..]),
        (0x162e, &hex!("e54e64016057754e0190e680e030e70dd2a27f107e271218aac2a2800bd2a47f107e271218aac2a41217ba75b2ff75b3ff75b4ff75b57e75b6fd121305c2a0903c057412f0a37434f0c2a2c2a475b37fd297c296c2b5c2b6c2b712180122")[..]),
        (0x10d0, &hex!("30040990e680e0440af0800790e680e04408f07fdc7e0512168c90e65d74fff090e65ff05391ef90e680e054f7f022")[..]),
        (0x1738, &hex!("a907ae14af158f828e83a3e064037017ad0119ed7001228f828e83e07c002ffdec3efeaf0580df7e007f0022")[..]),
        (0x168c, &hex!("8e318f3290e600e054187012e5322401ffe43531c313f531ef13f532801590e600e05418ffbf100be53225e0f532e53133f531e5321532ae31700215314e60051213ee80ee22")[..]),
        (0x13ee, &hex!("7400f58690fda57c05a3e582458370f922")[..]),
        (0x17ba, &hex!("e589540f4420f589438e1043878043d88075985075c050758df3758bf3c2acc2aed28e22")[..]),
        (0x0003, &hex!("8f993099fdc29922")[..]),
        (0x19c6, &hex!("3098fdaf99c29822")[..]),
        (0x16d2, &hex!("5391ef90e65d2290e740f0e490e68af090e68b04f0d3225391ef90e65f22850d82850c83a37402f022850f82850e83a37407f022")[..]),
        (0x1348, &hex!("90e605e054fdf0d20090e60b7403f090e61074a0f000000090e611f000000090e61274a2f000000090e6137420f000000090e61474e0f000000090e6157460f0e490e618f000000090e61af000000000000090e6047480f00000007402f00000007404f00000007406f00000007408f0000000e4f000000090e65f74fff0000000e490e65ef000000090e6497482f0000000f0000000f0000000f0000000e490e68df0c2a722")[..]),
        (0x1915, &hex!("90e68de0d39400400a12005690e600e04410f022")[..]),
        (0x0051, &hex!("d322")[..]),
        (0x19ce, &hex!("90e6bae0f519d322")[..]),
        (0x19e5, &hex!("e5191216d922")[..]),
        (0x19d6, &hex!("90e6bae0f518d322")[..]),
        (0x19eb, &hex!("e5181216d922")[..]),
        (0x19f5, &hex!("d322d322d322d322")[..]),
        (0x18eb, &hex!("c0e0c083c082d2011216d27401f0d082d083d0e032")[..]),
        (0x193c, &hex!("c0e0c083c0821216d27404f0d082d083d0e032c0e0c083c0821216d27402f0d082d083d0e032")[..]),
        (0x178f, &hex!("c0e0c083c08285100c85110d1216f085080e85090f1216fb7516007517401216d27410f0d082d083d0e032")[..]),
        (0x1900, &hex!("c0e0c083c082d2031216d27408f0d082d083d0e032")[..]),
        (0x1706, &hex!("c0e0c083c08290e680e030e71885080c85090d1216f085100e85110f1216fb7516027517001216d27420f0d082d083d0e032")[..]),
        (0x0032, &hex!("32")[..]),
        (0x10ff, &hex!("32")[..]),
        (0x13ff, &hex!("32")[..]),
        (0x19fd, &hex!("32")[..]),
        (0x1962, &hex!("c0e0c083c0821216e97404f0d082d083d0e032c0e0c083c0821216e97408f0d082d083d0e032")[..]),
        (0x18bc, &hex!("c0e0c083c08290e6497482f01216e97410f0d082d083d0e032")[..]),
        (0x19fe, &hex!("32")[..]),
        (0x1988, &hex!("c0e0c083c0821216e97440f0d082d083d0e032")[..]),
        (0x19ff, &hex!("323232323232323232323232323232323232323232323232323232")[..]),
        (0x199b, &hex!("90e6007410f0e04402f0e58e54f84401f58e22")[..]),
        (0x0000, &hex!("0214b8")[..]),
        (0x14b8, &hex!("787fe4f6d8fd7581950214ff")[..]),
        (0x11cc, &hex!("bb010689828a83e0225002e722bbfe02e32289828a83e49322bb010ce58229f582e5833af583e0225006e92582f8e622bbfe06e92582f8e222e58229f582e5833af583e49322bb010689828a83f0225002f722bbfe01f322f8bb010de58229f582e5833af583e8f0225006e92582c8f622bbfe05e92582c8f222ef8df0a4a8f0cf8cf0a428ce8df0a42efe22eb9ff5f0ea9e42f0e99d42f0e89c45f022d083d082f8e4937012740193700da3a393f8740193f5828883e4737402936860efa3a3a380df")[..]),
        (0x14c4, &hex!("020fa1e493a3f8e493a34003f68001f208dff48029e493a3f85407240cc8c333c4540f4420c8834004f456800146f6dfe4800b0102040810204080901846e47e019360bca3ff543f30e509541ffee493a360010ecf54c025e060a840b8e493a3fae493a3f8e493a3c8c582c8cac583caf0a3c8c582c8cac583cadfe9dee780be")[..]),
        (0x1867, &hex!("00")[..]),
        (0xe600, &hex!("00")[..]),
    ].iter() {
        handle.write_control(0x40, 160, *val, 0, *data, one_second)?;
    }

    Ok(())
}
