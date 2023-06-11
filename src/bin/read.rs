#![cfg(target_os = "linux")]

use easybench::bench;
use nix::libc::getppid;
use nix::libc::prctl;
use nix::libc::NT_PRSTATUS;
use nix::libc::PR_SET_DUMPABLE;
use nix::libc::PR_SET_PTRACER;
use nix::sys::personality;
use nix::sys::personality::Persona;
use nix::sys::ptrace::cont;
use nix::sys::ptrace::getsiginfo;
use nix::sys::ptrace::setoptions;
use nix::sys::ptrace::traceme;
use nix::sys::ptrace::Options;
use nix::sys::ptrace::Request;
use nix::sys::ptrace::RequestType;
use nix::sys::signal::Signal;
use nix::sys::signal::Signal::SIGTRAP;
use nix::sys::uio::pread;
use nix::sys::uio::process_vm_readv;
use nix::sys::uio::RemoteIoVec;
use nix::sys::wait::waitpid;
use nix::sys::wait::WaitPidFlag;
use nix::sys::wait::WaitStatus;
use nix::unistd::Pid;
use std::ffi::c_void;
use std::fs::File;
use std::io::IoSliceMut;
use std::os::fd::AsRawFd;
use std::os::unix::process::CommandExt;
use std::process::Command;
use zerocopy::AsBytes;
use zerocopy::FromBytes;

#[cfg(target_arch = "x86_64")]
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, AsBytes, FromBytes, Default)]
pub struct GpRegisters {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbp: u64,
    pub rbx: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub orig_rax: u64,
    pub rip: u64,
    pub cs: u64,
    pub eflags: u64,
    pub rsp: u64,
    pub ss: u64,
    pub fs_base: u64,
    pub gs_base: u64,
    pub ds: u64,
    pub es: u64,
    pub fs: u64,
    pub gs: u64,
}

#[cfg(target_arch = "aarch64")]
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, AsBytes, FromBytes, Default)]
pub struct GpRegisters {
    pub regs: [u64; 31],
    pub sp: u64,
    pub pc: u64,
    pub pstate: u64,
}

fn get_reg_set<T: AsBytes + FromBytes + Default>(pid: Pid, set: i32) -> nix::Result<T> {
    let mut data = T::default();
    let vec = nix::libc::iovec {
        iov_base: &mut data as *mut _ as *mut c_void,
        iov_len: std::mem::size_of::<T>(),
    };

    // SAFETY: Using FFI with the process trace API to read raw bytes.
    let err = unsafe {
        nix::libc::ptrace(
            Request::PTRACE_GETREGSET as RequestType,
            nix::libc::pid_t::from(pid),
            set,
            &vec as *const _ as *const c_void,
        )
    };

    nix::errno::Errno::result(err)?;
    Ok(data)
}

fn main() {
    let byte_num = 128 << 20;
    let mut cmd = Command::new("target/release/allocate");
    cmd.arg(format!("{byte_num}"));

    unsafe {
        cmd.pre_exec(|| {
            let persona = personality::get()?;
            personality::set(persona | Persona::ADDR_NO_RANDOMIZE)?;

            prctl(PR_SET_DUMPABLE, 1_u64);
            prctl(PR_SET_PTRACER, getppid() as u64);
            traceme().expect("can indicate tracing start");

            Ok(())
        });
    }

    let child = cmd.spawn().expect("can start allocating process");

    let child_pid = Pid::from_raw(child.id() as _);

    let status = waitpid(child_pid, Some(WaitPidFlag::WSTOPPED)).expect("can wait for the tracee");
    let sig_info = getsiginfo(child_pid).expect("can get signal info");
    println!("TRACER: wait status {status:x?}, signal info {sig_info:x?}");
    setoptions(child_pid, Options::PTRACE_O_EXITKILL).expect("can set ptrace options");

    cont(child_pid, Signal::SIGCONT).expect("can continue the tracee");

    let status = waitpid(child_pid, Some(WaitPidFlag::WSTOPPED)).expect("can wait for the tracee");
    let sig_info = getsiginfo(child_pid).expect("can get signal info");
    println!("TRACER: wait status {status:x?}, signal info {sig_info:x?}");

    assert!(status == WaitStatus::Stopped(child_pid, SIGTRAP));

    let gp_regs: GpRegisters =
        get_reg_set(child_pid, NT_PRSTATUS).expect("can read general registers");
    println!("TRACER: gp registers {gp_regs:x?}");

    let remote_ptr = {
        #[cfg(target_arch = "aarch64")]
        {
            gp_regs.regs[0]
        }

        #[cfg(target_arch = "x86_64")]
        {
            gp_regs.rax
        }
    };

    let file =
        File::open(format!("/proc/{child_pid}/mem")).expect("can open the memory pseudo-file");

    let mut read_size = 16;
    while read_size <= byte_num {
        let mut buf = vec![0u8; read_size];

        let pread_stats = bench(|| {
            pread(file.as_raw_fd(), buf.as_mut_slice(), remote_ptr as i64)
                .expect("can read the memory file")
        });
        let process_vm_read_stats = bench(|| {
            process_vm_readv(
                child_pid,
                &mut [IoSliceMut::new(buf.as_mut_slice()); 1],
                &[RemoteIoVec {
                    base: remote_ptr as usize,
                    len: read_size,
                }],
            )
            .expect("can read remote process memory")
        });

        println!(
            "Size: {read_size}, pread/process_vm_read: {:.02}",
            pread_stats.ns_per_iter / process_vm_read_stats.ns_per_iter
        );

        read_size *= 2;
    }

    cont(child_pid, Signal::SIGCONT).expect("can continue the tracee");
}
