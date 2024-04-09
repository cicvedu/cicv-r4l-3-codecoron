//! Rust for linux completion_rust demo

use kernel::prelude::*;
use kernel::bindings;
use kernel::sync::Mutex;
use kernel::{chrdev, file};

module!{
    type: CompletionCDevRust,
    name: "completion_rust",
    author: "coderon",
    description: "Rust for linux completion_rust demo",
    license: "GPL",
}

const GLOBALMEM_SIZE: usize = 0x1000;

static GLOBALMEM_BUF: Mutex<[u8;GLOBALMEM_SIZE]> = unsafe {
    Mutex::new([0u8;GLOBALMEM_SIZE])
};

struct RustFile {
    #[allow(dead_code)]
    inner: &'static Mutex<[u8;GLOBALMEM_SIZE]>,
    _completion: bindings::completion,
}

unsafe impl Send for RustFile {}
unsafe impl Sync for RustFile {}

#[vtable]
impl file::Operations for RustFile {
    type Data = Box<Self>;

    fn open(_shared: &(), _file: &file::File) -> Result<Box<Self>> {
        /*
        1. init_completion()---只需要init completion ,chardev的init和rust_chrdev.rs一样
        -> 会调用__init_swait_queue_head() 
         */
        let _completion = unsafe {
            let mut key = bindings::lock_class_key {};
            let mut _completion = bindings::completion::default();
            _completion.done = 0;
            // todo!() name dynamic generate
            bindings::__init_swait_queue_head(&mut _completion.wait, &1_i8, &mut key);
            _completion
        };
        Ok(
            Box::try_new(RustFile {
                inner: &GLOBALMEM_BUF,
                _completion,
            })?
        )
    }

    fn write(_this: &Self,_file: &file::File,_reader: &mut impl kernel::io_buffer::IoBufferReader,_offset:u64,) -> Result<usize> {
        
        // Writes data from the caller's buffer to this file.
        let _buf = &mut _this.inner.lock();
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

        unsafe{
            let mut _completion = _this._completion;
            bindings::complete(&mut _completion);
        }

        Ok(len)
    }

    fn read(_this: &Self,_file: &file::File,_writer: &mut impl kernel::io_buffer::IoBufferWriter,_offset:u64,) -> Result<usize> {

        unsafe{
            let mut _completion = _this._completion;
            bindings::wait_for_completion(&mut _completion);
        }

        // Reads data from this file to the caller's buffer.
        let _data = &mut _this.inner.lock();
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



struct CompletionCDevRust{
        _cdev: Pin<Box<chrdev::Registration<2>>>,
        // _completion: bindings::completion,
}
unsafe impl Send for CompletionCDevRust {}
unsafe impl Sync for CompletionCDevRust {}



impl kernel::Module for CompletionCDevRust{
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("in init\n");

         let mut chrdev_reg = chrdev::Registration::new_pinned(_name, 0, _module)?;

         // Register the same kind of device twice, we're just demonstrating
         // that you can use multiple minors. There are two minors in this case
         // because its type is `chrdev::Registration<2>`
         chrdev_reg.as_mut().register::<RustFile>()?;
         chrdev_reg.as_mut().register::<RustFile>()?;


        Ok(CompletionCDevRust{
            _cdev: chrdev_reg,
        })
    }
}

impl Drop for CompletionCDevRust{
    fn drop(&mut self){
        pr_info!("in drop\n");
    }
}