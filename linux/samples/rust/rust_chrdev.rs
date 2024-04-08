// SPDX-License-Identifier: GPL-2.0

//! Rust character device sample.

use core::result::Result::{Err, Ok};

use kernel::prelude::*;
use kernel::sync::Mutex;
use kernel::{chrdev, file};

const GLOBALMEM_SIZE: usize = 0x1000;

module! {
    type: RustChrdev,
    name: "rust_chrdev",
    author: "Rust for Linux Contributors",
    description: "Rust character device sample",
    license: "GPL",
}

static GLOBALMEM_BUF: Mutex<[u8;GLOBALMEM_SIZE]> = unsafe {
    Mutex::new([0u8;GLOBALMEM_SIZE])
};

struct RustFile {
    #[allow(dead_code)]
    inner: &'static Mutex<[u8;GLOBALMEM_SIZE]>,
}

#[vtable]
impl file::Operations for RustFile {
    type Data = Box<Self>;

    fn open(_shared: &(), _file: &file::File) -> Result<Box<Self>> {
        Ok(
            Box::try_new(RustFile {
                inner: &GLOBALMEM_BUF
            })?
        )
    }

    fn write(_this: &Self,_file: &file::File,_reader: &mut impl kernel::io_buffer::IoBufferReader,_offset:u64,) -> Result<usize> {
        
        // Writes data from the caller's buffer to this file.
        let buf = &mut _this.inner.lock();
        // 约定，将从buf读取的字节数返回
        // 上层会将return 作为下次调用的offset , 持续调用write
        let mut len = _reader.len();
        pr_info!("in write\n");
        pr_info!("offset {}\n",_offset);
        pr_info!("len {}\n",len);
        if len > GLOBALMEM_SIZE {
            len = GLOBALMEM_SIZE;
        }
        // _reader.read_slice(&mut **buf)?;  // 编译通过，但是写数据出现 address error
        _reader.read_slice(&mut buf[_offset as usize ..len])?;
        Ok(len)
    }

    fn read(_this: &Self,_file: &file::File,_writer: &mut impl kernel::io_buffer::IoBufferWriter,_offset:u64,) -> Result<usize> {
        // Reads data from this file to the caller's buffer.
        let data = &mut _this.inner.lock();
        // 约定要将写入buf的字节数返回
        // offset 用来判断读取进度---- 上层调用会持续调用read() 
        if _offset as usize >= GLOBALMEM_SIZE {
            return Ok(0);
        }
        let _len = _writer.len();
        pr_info!("in read\n");
        pr_info!("offset {}\n",_offset);
        pr_info!("len {}\n",_len);
        _writer.write_slice(&data[_offset as usize..])?;
        Ok(_len)
    }
}

struct RustChrdev {
    _dev: Pin<Box<chrdev::Registration<2>>>,
}

impl kernel::Module for RustChrdev {
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust character device sample (init)\n");

        let mut chrdev_reg = chrdev::Registration::new_pinned(name, 0, module)?;

        // Register the same kind of device twice, we're just demonstrating
        // that you can use multiple minors. There are two minors in this case
        // because its type is `chrdev::Registration<2>`
        chrdev_reg.as_mut().register::<RustFile>()?;
        chrdev_reg.as_mut().register::<RustFile>()?;

        Ok(RustChrdev { _dev: chrdev_reg })
    }
}

impl Drop for RustChrdev {
    fn drop(&mut self) {
        pr_info!("Rust character device sample (exit)\n");
    }
}
