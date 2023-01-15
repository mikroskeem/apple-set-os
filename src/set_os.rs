use uefi::proto::Protocol;
use uefi::{unsafe_guid, CStr8, Status};

#[repr(C)]
#[unsafe_guid("c5c5da95-7d5c-45e6-b2f1-3fd52bb10077")]
#[derive(Protocol)]
#[allow(non_snake_case)]
pub struct AppleSetOS {
    SetOsVersion: extern "efiapi" fn(this: &AppleSetOS, version: &CStr8) -> Status,
    SetOsVendor: extern "efiapi" fn(this: &AppleSetOS, vendor: &CStr8) -> Status,
}

impl AppleSetOS {
    pub fn set_os_version(&self, version: &str) -> Status {
        let version = CStr8::from_bytes_with_nul(version.as_bytes()).unwrap();

        (self.SetOsVersion)(self, version)
    }

    pub fn set_os_vendor(&self, vendor: &str) -> Status {
        let vendor = CStr8::from_bytes_with_nul(vendor.as_bytes()).unwrap();

        (self.SetOsVendor)(self, vendor)
    }
}
