#![no_main]
#![no_std]
#![feature(abi_efiapi)]
#![feature(negative_impls)]
#![allow(stable_features)]

mod set_os;

use log::{error, trace};
use uefi::prelude::*;
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode, FileType};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::{MemoryType, SearchType};
use uefi::{Identify, Result};

use crate::set_os::AppleSetOS;

static DESIRED_OS_VENDOR: &str = "Apple Inc.";
static DESIRED_OS_VERSION: &str = "Mac OS X 10.15";

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    let boot_svc = system_table.boot_services();
    trace!("APPLE_SET_OS_GUID={}", &AppleSetOS::GUID);

    // Set OS vendor & version
    if let Err(err) = apple_set_os(boot_svc, DESIRED_OS_VENDOR, DESIRED_OS_VERSION) {
        error!("Failed to set OS vendor & version: {:?}", err);
    }

    // Chainload
    trace!("Chainloading");
    if let Err(err) = chainload_image(boot_svc) {
        error!("Failed to chainload: {:?}", err);

        boot_svc.stall(5_000_000);
        return Status::UNSUPPORTED;
    }

    Status::SUCCESS
}

fn apple_set_os(boot_svc: &BootServices, vendor: &str, version: &str) -> Result<()> {
    let set_os_handle_buffers =
        boot_svc.locate_handle_buffer(SearchType::ByProtocol(&AppleSetOS::GUID))?;

    let set_os_handle = set_os_handle_buffers.handles().first().unwrap();

    trace!("AppleSetOS handle present");
    let protocol = boot_svc.open_protocol_exclusive::<AppleSetOS>(*set_os_handle)?;

    trace!("Desired vendor='{}' version='{}'", vendor, version);

    let vendor_result = protocol.set_os_vendor(vendor);
    if !vendor_result.is_success() {
        // TODO: return proper error
        error!("Unable to set OS vendor: {:?}", vendor_result);
    }

    let os_result = protocol.set_os_version(version);
    if !os_result.is_success() {
        // TODO: return proper error
        error!("Unable to set OS version: {:?}", os_result);
    }

    Ok(())
}

fn chainload_image(boot_svc: &BootServices) -> Result<()> {
    let current_image = boot_svc.image_handle();
    let loaded_image_proto = boot_svc.open_protocol_exclusive::<LoadedImage>(current_image)?;
    let loaded_image_device = loaded_image_proto.device();

    let mut file_protocol =
        boot_svc.open_protocol_exclusive::<SimpleFileSystem>(loaded_image_device)?;
    let mut vol = file_protocol.open_volume()?;

    let file_path = cstr16!("\\EFI\\Boot\\bootx64_original.efi");
    let mut file = vol.open(file_path, FileMode::Read, FileAttribute::READ_ONLY)?;

    let mut file_info_buf = [0; 128];
    let file_info = file.get_info::<FileInfo>(&mut file_info_buf).unwrap();

    let file_size = file_info.file_size() as usize;
    let pool_addr = boot_svc.allocate_pool(MemoryType::LOADER_DATA, file_size)?;
    let file_data: &mut [u8] =
        unsafe { core::slice::from_raw_parts_mut(pool_addr as *mut u8, file_size) };

    match file.into_type()? {
        FileType::Regular(mut regular_file) => {
            regular_file.read(file_data).unwrap();
        }
        FileType::Dir(_) => {
            // TODO: turn this into a constant
            panic!(
                "unexpected directory at path '{}'",
                "\\EFI\\Boot\\bootx64_original.efi"
            );
        }
    }

    /*
    let new_image = boot_svc.load_image(
        current_image,
        uefi::table::boot::LoadImageSource::FromFilePath {
            file_path: file.into(),
            from_boot_manager: false,
        },
    )?;
    */

    let new_image = boot_svc.load_image(
        current_image,
        uefi::table::boot::LoadImageSource::FromBuffer {
            file_path: None,
            buffer: file_data,
        },
    )?;

    boot_svc.free_pool(pool_addr)?;
    boot_svc.start_image(new_image)?;

    Ok(())
}
