//! Rust for linux completion_rust demo

use kernel::task;
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

struct CompletionStruct(bindings::completion);

unsafe impl Send for CompletionStruct {}
unsafe impl Sync for CompletionStruct {}

static mut GLOBALMEM_COMP: Option<CompletionStruct> = None;

struct RustFile {
    #[allow(dead_code)]
    inner: &'static Mutex<[u8;GLOBALMEM_SIZE]>,
}

unsafe impl Send for RustFile {}
unsafe impl Sync for RustFile {}

#[vtable]
impl file::Operations for RustFile {
    type Data = Box<Self>;

    fn open(_shared: &(), _file: &file::File) -> Result<Box<Self>> {
        pr_info!("open in invoked");
        Ok(
            Box::try_new(RustFile {
                inner: &GLOBALMEM_BUF,
            })?
        )
    }

    fn write(_this: &Self,_file: &file::File,_reader: &mut impl kernel::io_buffer::IoBufferReader,_offset:u64,) -> Result<usize> {
        pr_info!("write is invoked\n");
        pr_info!("process {} awakening the readers...\n", task::Task::current().pid());
        // Writes data from the caller's buffer to this file.
        let _buf = &mut _this.inner.lock();
        let mut len = _reader.len();
        if len > GLOBALMEM_SIZE {
            len = GLOBALMEM_SIZE;
        }

        unsafe {
            match &mut GLOBALMEM_COMP {
                Some(ref mut completion) => {
                    let ptr = &mut completion.0 as *mut bindings::completion;
                    bindings::complete(ptr);
                }
                None => {
                    pr_info!("None\n");
                }
            }
        }

        Ok(len)
    }

    fn read(_this: &Self,_file: &file::File,_writer: &mut impl kernel::io_buffer::IoBufferWriter,_offset:u64,) -> Result<usize> {
        pr_info!("read is invoked\n");
        pr_info!("process {} is going to sleep\n", task::Task::current().pid());
        unsafe {
            match &mut GLOBALMEM_COMP {
                Some(ref mut completion) => {
                    let ptr = &mut completion.0 as *mut bindings::completion;
                    bindings::wait_for_completion(ptr);
                }
                None => {
                    pr_info!("None\n");
                }
            }
        }
        pr_info!("awoken {}\n", task::Task::current().pid());
        // Reads data from this file to the caller's buffer.
        let _data = &mut _this.inner.lock();
        if _offset as usize >= GLOBALMEM_SIZE {
            return Ok(0);
        }
        let _len = _writer.len();
        pr_info!("offset {}\n",_offset);
        pr_info!("len {}\n",_len);
        // _writer.write_slice(&data[_offset as usize..])?;
        Ok(_len)
    }
}



struct CompletionCDevRust{
        _cdev: Pin<Box<chrdev::Registration<2>>>,
}
unsafe impl Send for CompletionCDevRust {}
unsafe impl Sync for CompletionCDevRust {}



impl kernel::Module for CompletionCDevRust{
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("in init\n");
        //----code1---todo--why bug?
        // unsafe {
        //     let mut _completion = bindings::completion::default();
        //     let ptr = &mut _completion;
        //     bindings::init_completion(ptr);
        //     GLOBALMEM_COMP = Some(CompletionStruct(_completion)); 
        // };

        // // --- code2
        unsafe {
            let mut _completion = bindings::completion::default();
            GLOBALMEM_COMP = Some(CompletionStruct(_completion));

            match &mut GLOBALMEM_COMP {
                Some(ref mut _completion) =>{
                    bindings::init_completion(&mut _completion.0);
                },
                None =>{
                    pr_info!("None\n");
                }
            }
        };

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