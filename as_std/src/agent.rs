use core::fmt::Display;

use alloc::{
    format,
    string::{String, ToString},
};

pub type FaaSFuncResult<T> = Result<DataBuffer<T>, FaaSFuncError>;

#[derive(Debug)]

pub struct FaaSFuncError {
    msg: String,
}

impl<T> From<T> for FaaSFuncError
where
    T: Display,
{
    fn from(value: T) -> Self {
        Self {
            msg: value.to_string(),
        }
    }
}

impl FaaSFuncError {
    pub fn msg(&self) -> String {
        format!("user function error: {}", self.msg)
    }
}

#[cfg(not(feature = "file-based"))]
mod refer_based_impl {
    use core::{alloc::Layout, borrow::Borrow, mem::ManuallyDrop};

    use alloc::{boxed::Box, string::String};
    use as_hostcall::Verify;

    use crate::{libos::libos, println};

    #[derive(Debug)]
    pub struct DataBuffer<T> {
        inner: ManuallyDrop<Box<T>>,

        used: bool,
    }

    impl<T> DataBuffer<T>
    where
        T: Verify,
    {
        pub fn new() -> Self
        where
            T: Default,
        {
            Self::with_slot(String::new())
        }

        pub fn with_slot(slot: String) -> Self
        where
            T: Default,
        {
            let l: Layout = Layout::new::<T>();

            let p = if l.size() == 0 {
                unsafe {
                    alloc::alloc::alloc(Layout::from_size_align(4, 4).unwrap()) as usize as *mut T
                }
            } else {
                let fingerprint = T::__fingerprint();

                libos!(buffer_alloc(&slot, l, fingerprint)).expect("alloc failed.") as *mut T

                // let val = T::default();
                // println!("will write addr=0x{:x}", addr as usize);
                // unsafe { core::ptr::write(addr, val) };
            };

            let inner = unsafe { Box::from_raw(p) };

            Self {
                inner: ManuallyDrop::new(inner),
                used: false,
            }
        }

        pub fn from_buffer() -> Option<Self> {
            Self::from_buffer_slot(String::new())
        }

        pub fn from_buffer_slot(slot: String) -> Option<Self> {
            let buffer_meta: Option<(usize, u64)> = libos!(access_buffer(&slot));

            buffer_meta.map(|(raw_ptr, fingerprint)| {
                if fingerprint != T::__fingerprint() {
                    println!("wrong data type, {}, {}", fingerprint, T::__fingerprint());
                    panic!("");
                };

                let inner = unsafe { Box::from_raw(raw_ptr as *mut T) };

                Self {
                    inner: ManuallyDrop::new(inner),
                    used: false,
                }
            })
        }
    }

    impl<T> Default for DataBuffer<T>
    where
        T: Default + Verify,
    {
        fn default() -> DataBuffer<T> {
            Self::new()
        }
    }

    impl<T> core::ops::Deref for DataBuffer<T>
    where
        T: Verify,
    {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.inner.borrow()
        }
    }

    impl<T> core::ops::DerefMut for DataBuffer<T>
    where
        T: Verify,
    {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.inner.as_mut()
        }
    }

    impl<T> From<T> for DataBuffer<T>
    where
        T: Default + Verify,
    {
        fn from(value: T) -> Self {
            let mut t = DataBuffer::<T>::default();
            *t = value;
            t
        }
    }

    impl<T> Drop for DataBuffer<T> {
        fn drop(&mut self) {
            if self.used {
                let ptr = Box::into_raw(unsafe { ManuallyDrop::take(&mut self.inner) });
                // println!("drop DataBuffer val: 0x{:x}", ptr as usize);
                libos!(buffer_dealloc(ptr as usize, Layout::new::<T>()));
            }
        }
    }
}

#[cfg(feature = "file-based")]
mod file_based_impl {
    use alloc::{format, string::String};
    use core::{borrow::Borrow, fmt::Write};
    use as_hostcall::Verify;
    use serde::{Deserialize, Serialize};

    use crate::fs::File;

    #[derive(Debug)]
    pub struct DataBuffer<T>
    where
        T: Serialize,
    {
        inner: T,
        slot: String,
        used: bool,
    }

    impl<T> DataBuffer<T>
    where
        T: Verify + Serialize + for<'a> Deserialize<'a> + Default,
    {
        pub fn new() -> Self
        where
            T: Default,
        {
            Self::with_slot(String::new())
        }

        pub fn with_slot(slot: String) -> Self
        where
            T: Default,
        {
            Self {
                inner: T::default(),
                slot,
                used: false,
            }
        }

        pub fn from_buffer() -> Option<Self> {
            Self::from_buffer_slot(String::new())
        }

        pub fn from_buffer_slot(slot: String) -> Option<Self>
        where
            T: Default,
        {
            use alloc::vec::Vec;

            use crate::io::Read;

            let mut content = if let Ok(file) = File::open(&format!("{}.imd", slot)) {
                file
            } else {
                return None;
            };

            let mut result = Self {
                inner: Default::default(),
                slot: slot.clone(),
                used: true,
            };

            let mut buffer = Vec::new();
            content
                .read_to_end(&mut buffer)
                .expect("read imd file failed.");
            result.inner =
                serde_json::from_slice(&buffer).expect(&format!("deserialize {} failed.", &slot));

            Some(result)
        }
    }

    impl<T> Default for DataBuffer<T>
    where
        T: Default + Verify + Serialize + for<'a> Deserialize<'a>,
    {
        fn default() -> DataBuffer<T> {
            Self::new()
        }
    }

    impl<T> core::ops::Deref for DataBuffer<T>
    where
        T: Verify + Serialize,
    {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.inner.borrow()
        }
    }

    impl<T> core::ops::DerefMut for DataBuffer<T>
    where
        T: Verify + Serialize,
    {
        fn deref_mut(&mut self) -> &mut Self::Target {
            #[cfg(feature = "refer-based")]
            {
                self.inner.as_mut()
            }
            #[cfg(feature = "file-based")]
            {
                &mut self.inner
            }
        }
    }

    impl<T> From<T> for DataBuffer<T>
    where
        T: Default + Verify + Serialize + for<'a> Deserialize<'a>,
    {
        fn from(value: T) -> Self {
            let mut t = DataBuffer::<T>::default();
            *t = value;
            t
        }
    }

    impl<T> Drop for DataBuffer<T>
    where
        T: Serialize,
    {
        fn drop(&mut self) {
            if !self.used {
                let mut content =
                    File::create(&format!("{}.imd", &self.slot)).expect("create imd file failed.");
                let data_str = serde_json::to_string(&self.inner).expect("serialize failed.");

                content.write_str(&data_str).expect("write failed.");
            }
        }
    }
}

#[cfg(not(feature = "file-based"))]
pub use refer_based_impl::*;

#[cfg(feature = "file-based")]
pub use file_based_impl::*;
