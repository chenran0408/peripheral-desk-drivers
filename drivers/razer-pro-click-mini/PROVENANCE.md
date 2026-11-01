# Razer Pro Click Mini

This package matches only the Razer receiver PID 1532:009A and a 91-byte Feature interface. It reads the battery value through the command channel documented by Voltide's Razer provider. Charging, DPI, polling, lighting, remapping and macros are not exposed because Voltide's provider does not establish model-specific support for them.

The adapter is an untested preview. The receiver PID is kept exact so this package cannot claim support for other Razer models or shared receiver identities.
