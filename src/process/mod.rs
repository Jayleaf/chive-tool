pub mod process {

    use std::ffi::c_void;
    use std::io;
    use std::mem;
    use std::mem::MaybeUninit;
    use std::ptr::NonNull;
    use winapi::shared::minwindef::HMODULE;
    use winapi::shared::minwindef::{DWORD, FALSE};
    use winapi::um::winnt;

    pub struct Process {
        pub pid: u32,
        pub handle: NonNull<c_void>,
    }

    impl Process {
        pub fn open(pid: u32) -> io::Result<Self> {
            NonNull::new(unsafe {
                winapi::um::processthreadsapi::OpenProcess(
                    winnt::PROCESS_QUERY_INFORMATION | winnt::PROCESS_VM_READ,
                    FALSE,
                    pid,
                )
            })
            .map(|handle| Self { pid, handle })
            .ok_or_else(io::Error::last_os_error)
        }
        pub fn name(&self) -> io::Result<String> {
            let mut module = MaybeUninit::<HMODULE>::uninit();
            let mut size = 0;
            // SAFETY: the pointer is valid and the size is correct.
            if unsafe {
                winapi::um::psapi::EnumProcessModules(
                    self.handle.as_ptr(),
                    module.as_mut_ptr(),
                    mem::size_of::<HMODULE>() as u32,
                    &mut size,
                )
            } == FALSE
            {
                return Err(io::Error::last_os_error());
            }

            // SAFETY: the call succeeded, so module is initialized.
            let module = unsafe { module.assume_init() };
            let mut buffer = Vec::<u8>::with_capacity(64);
            // SAFETY: the handle, module and buffer are all valid.
            let length = unsafe {
                winapi::um::psapi::GetModuleBaseNameA(
                    self.handle.as_ptr(),
                    module,
                    buffer.as_mut_ptr().cast(),
                    buffer.capacity() as u32,
                )
            };
            if length == 0 {
                return Err(io::Error::last_os_error());
            }

            // SAFETY: the call succeeded and length represents bytes.
            unsafe { buffer.set_len(length as usize) };
            Ok(String::from_utf8(buffer).unwrap())
        }

        pub fn memory_regions(&self) -> Vec<winapi::um::winnt::MEMORY_BASIC_INFORMATION> {
            let mut base = 0;
            let mut regions = Vec::new();
            let mut info = MaybeUninit::uninit();

            loop {
                // SAFETY: the info structure points to valid memory.
                let written = unsafe {
                    winapi::um::memoryapi::VirtualQueryEx(
                        self.handle.as_ptr(),
                        base as *const _,
                        info.as_mut_ptr(),
                        mem::size_of::<winapi::um::winnt::MEMORY_BASIC_INFORMATION>(),
                    )
                };
                if written == 0 {
                    break regions;
                }
                // SAFETY: a non-zero amount was written to the structure
                let info = unsafe { info.assume_init() };
                base = info.BaseAddress as usize + info.RegionSize;
                regions.push(info);
            }
        }

        pub fn read_memory(&self, addr: usize, n: usize) -> io::Result<Vec<u8>> {
            let mut buffer = Vec::<u8>::with_capacity(n);
            let mut read = 0;

            // SAFETY: the buffer points to valid memory, and the buffer size is correctly set.
            if unsafe {
                winapi::um::memoryapi::ReadProcessMemory(
                    self.handle.as_ptr(),
                    addr as *const _,
                    buffer.as_mut_ptr().cast(),
                    buffer.capacity(),
                    &mut read,
                )
            } == FALSE
            {
                Err(io::Error::last_os_error())
            } else {
                // SAFETY: the call succeeded and `read` contains the amount of bytes written.
                unsafe { buffer.set_len(read as usize) };
                Ok(buffer)
            }
        }

        pub fn value_at(&self, addr: usize) -> io::Result<u32> {
            let mut buffer = MaybeUninit::<u32>::uninit();
            let mut read = 0;

            // SAFETY: the buffer points to valid memory.
            if unsafe {
                winapi::um::memoryapi::ReadProcessMemory(
                    self.handle.as_ptr(),
                    addr as *const _,
                    buffer.as_mut_ptr().cast(),
                    mem::size_of::<u32>(),
                    &mut read,
                )
            } == FALSE
            {
                Err(io::Error::last_os_error())
            } else {
                // SAFETY: the call succeeded and `read` contains the amount of bytes written.
                Ok(unsafe { buffer.assume_init() })
            }
        }
    }

    impl Drop for Process {
        fn drop(&mut self) {
            unsafe { winapi::um::handleapi::CloseHandle(self.handle.as_mut()) };
        }
    }

    pub fn enum_proc() -> io::Result<Vec<u32>> {
        let mut pids = Vec::<DWORD>::with_capacity(1024);
        let mut size = 0;
        if unsafe {
            winapi::um::psapi::EnumProcesses(
                pids.as_mut_ptr(),
                (pids.capacity() * mem::size_of::<DWORD>()) as u32,
                &mut size,
            )
        } == FALSE
        {
            return Err(io::Error::last_os_error());
        }

        let count = size as usize / mem::size_of::<DWORD>();
        unsafe { pids.set_len(count) };
        Ok(pids)
    }
}
