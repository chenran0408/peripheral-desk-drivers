# HECATE G2 Pro vendor components

This package carries the x64 files needed for the G2 Pro's original audio effects and the existing control SDK. The Windows driver payload is taken from the user-provided HECATE application extraction at `data/deviceresource/g2pro/driver`; the SDK is taken from the user-provided installed reference at `HECATE G2 PRO/CPL/Driver/x64/vista/osConfLib.dll`. The package includes only the selected matching driver and APO revisions, not the HECATE installer, control panel, x86 payloads, unrelated device packages, or firmware/update files.

The package metadata declares every file by its archive path and SHA-256. These declarations are included in the signed driver catalog and checked again against the ZIP before installation.

| Name | Included file | Source revision | SHA-256 |
| --- | --- | --- | --- |
| `sdk` | `components/sdk/osConfLib.dll` | Installed HECATE control SDK reference | `576c1306133a33b975d7ecf25a2b7078052b2566e3705715c247d1e1a1ef2947` |
| `driver-inf` | `components/windows/HECATE_HD_AUDIO_DEVICE-Driver-V11.2.1.0/HECATE_HD_AUDIO_DEVICE.Inf` | HECATE audio driver 11.2.1.0 | `83d30e9bd1f54cad946dbd7c352b20fe76f73459cf606ba1160005a7e4ef5e71` |
| `driver-cat` | `components/windows/HECATE_HD_AUDIO_DEVICE-Driver-V11.2.1.0/HECATE_HD_AUDIO_DEVICE.cat` | HECATE audio driver 11.2.1.0 | `b30df4c0ca7812d0ff87228018e701902bebe29bcf6259dee0017777009780f1` |
| `driver-sys` | `components/windows/HECATE_HD_AUDIO_DEVICE-Driver-V11.2.1.0/x64/HECATE_HD_AUDIO_DEVICE.sys` | HECATE audio driver 11.2.1.0, amd64 | `f43f4a68ca2400214bd03e899e8ba0fc1c65b6edd292652735793cd150177fbd` |
| `extension-inf` | `components/windows/CM6646X1_HECATE_HD_AUDIO_DEVICE_APO_V2.1.0.2/01_EXTENSION/HECATE_HD_AUDIO_DEVICE_Ext.inf` | C-Media HECATE extension 1.1.0.2, amd64 | `8021fbbee79d05682b54d9e326f9ab9d56f2425302067e24ab42e8f333fb5ba7` |
| `extension-cat` | `components/windows/CM6646X1_HECATE_HD_AUDIO_DEVICE_APO_V2.1.0.2/01_EXTENSION/CMExt.cat` | C-Media HECATE extension 1.1.0.2 | `6615999ff1fe13ef37a3856460cadae0398947db75c50c5b14994172461f1d96` |
| `apo-inf` | `components/windows/CM6646X1_HECATE_HD_AUDIO_DEVICE_APO_V2.1.0.2/02_APO/HECATE_HD_AUDIO_DEVICE_APO.inf` | C-Media HECATE APO 2.1.0.2, amd64 | `61770e424c14b99a5963219bd04b797d5792e4b5ca25e60dbde994c09c3266e6` |
| `apo-cat` | `components/windows/CM6646X1_HECATE_HD_AUDIO_DEVICE_APO_V2.1.0.2/02_APO/CMAPO.cat` | C-Media HECATE APO 2.1.0.2 | `11b07692131a13cbf3f10de427dbaef63a498707ab47a3e0cfaa0ca13de66eb9` |
| `apo-engine` | `components/windows/CM6646X1_HECATE_HD_AUDIO_DEVICE_APO_V2.1.0.2/02_APO/CmApoIce.dll` | C-Media HECATE APO 2.1.0.2, amd64 | `17975053ccb0b1f8f6004246cad80f396a7dc151ddea366916aff20a86ed6de6` |

## Hardware and signature evidence

The selected base INF matches `USB\VID_35BB&PID_A100&MI_00`. The extension INF also matches PID `A100` and declares the APO software component. This is the newest matching x64 pair found in the supplied extraction: driver 11.2.1.0 with APO 2.1.0.2. The installed reference tree contains an older 2024 driver family and was used only for the SDK DLL.

The driver's SYS and CAT were reported as Authenticode-valid with the Microsoft Windows Hardware Compatibility Publisher signature. `CMExt.cat` and `CMAPO.cat` were reported as validly signed by C-MEDIA ELECTRONICS INC. The APO engine DLL has no embedded signature; it is covered by the APO package catalog. `osConfLib.dll` is unsigned. These signatures establish file/package integrity and publisher identity, not redistribution permission.

## Vendor attribution and license status

The SDK and Windows audio/extension/APO files are proprietary vendor binaries and are not covered by this repository's AGPL license. The maintainer reports having Edifier HECATE developer-program authorization. Inspection of the supplied extracted driver files and installed reference tree found no EULA, license text, or public redistribution grant, so this note does not invent or quote vendor license terms. The Windows catalog signatures establish package integrity and publisher identity, not copyright ownership or license scope.

The root `LICENSE` in a package covers this project's plugin source and does not relicense the vendor components. Do not modify or strip the vendor signatures/catalogs. The application must stage and install the signed driver packages through its normal Windows driver installation flow.
