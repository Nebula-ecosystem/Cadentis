use libc::{
    AF_INET, AF_INET6, O_CREAT, O_NONBLOCK, O_RDONLY, O_TRUNC, O_WRONLY, SHUT_RD, SHUT_RDWR,
    SHUT_WR, SO_REUSEADDR, SOCK_STREAM, SOL_SOCKET, accept, bind, c_char, c_int, close, connect,
    getsockname, listen, mkdir, mode_t, open, read, setsockopt, shutdown, sockaddr, sockaddr_in,
    sockaddr_in6, sockaddr_storage, socket, socklen_t, write,
};
use std::ffi::c_uint;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::os::fd::RawFd;
use std::{io, mem};

pub(crate) const OPENFLAGS: i32 = O_RDONLY | O_NONBLOCK;
pub(crate) const CREATEFLAGS: i32 = O_WRONLY | O_CREAT | O_TRUNC | O_NONBLOCK;

pub(crate) fn sys_read(fd: RawFd, buffer: &mut [u8]) -> isize {
    unsafe { read(fd, buffer.as_mut_ptr() as *mut _, buffer.len()) }
}

pub(crate) fn sys_write(fd: RawFd, buffer: &[u8]) -> isize {
    unsafe { write(fd, buffer.as_ptr() as *mut _, buffer.len()) }
}

pub(crate) fn sys_close(fd: RawFd) {
    unsafe { close(fd) };
}

pub(crate) fn sys_open(path: *const c_char, flags: i32, mode: mode_t) -> RawFd {
    unsafe { open(path, flags, mode as c_uint) }
}

pub(crate) fn sys_mkdir(path: *const c_char, mode: mode_t) -> RawFd {
    unsafe { mkdir(path, mode) }
}

pub(crate) fn sys_accept(fd: RawFd) -> io::Result<(RawFd, SocketAddr)> {
    let mut storage: sockaddr_storage = unsafe { mem::zeroed() };
    let mut len = mem::size_of::<sockaddr_storage>() as socklen_t;

    let client_fd = unsafe { accept(fd, &mut storage as *mut _ as *mut sockaddr, &mut len) };

    if client_fd < 0 {
        return Err(io::Error::last_os_error());
    }

    let addr = sockaddr_storage_to_socketaddr(&storage)?;
    Ok((client_fd, addr))
}

pub(crate) fn sys_sockname(fd: RawFd) -> io::Result<SocketAddr> {
    let mut storage: sockaddr_storage = unsafe { mem::zeroed() };
    let mut len = mem::size_of::<sockaddr_storage>() as socklen_t;
    let result = unsafe { getsockname(fd, &mut storage as *mut _ as *mut sockaddr, &mut len) };

    if result < 0 {
        return Err(io::Error::last_os_error());
    }

    sockaddr_storage_to_socketaddr(&storage)
}

pub(crate) fn sockaddr_storage_to_socketaddr(storage: &sockaddr_storage) -> io::Result<SocketAddr> {
    match storage.ss_family as c_int {
        AF_INET => {
            let addr = unsafe { &*(storage as *const _ as *const sockaddr_in) };
            let ip = Ipv4Addr::from(u32::from_be(addr.sin_addr.s_addr));
            let port = u16::from_be(addr.sin_port);

            Ok(SocketAddr::V4(SocketAddrV4::new(ip, port)))
        }
        AF_INET6 => {
            let addr = unsafe { &*(storage as *const _ as *const sockaddr_in6) };
            let ip = Ipv6Addr::from(addr.sin6_addr.s6_addr);
            let port = u16::from_be(addr.sin6_port);

            Ok(SocketAddr::V6(SocketAddrV6::new(
                ip,
                port,
                addr.sin6_flowinfo,
                addr.sin6_scope_id,
            )))
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported address family",
        )),
    }
}

pub(crate) fn sys_socket(domain: u8) -> io::Result<RawFd> {
    let fd = unsafe { socket(domain as c_int, SOCK_STREAM | O_NONBLOCK, 0) };

    if fd < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(fd)
    }
}

pub(crate) fn sys_bind(fd: RawFd, addr: &sockaddr_storage, len: socklen_t) -> io::Result<()> {
    let rc = unsafe { bind(fd, addr as *const _ as *const sockaddr, len) };

    if rc < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub(crate) fn sys_listen(fd: RawFd) -> io::Result<()> {
    let rc = unsafe { listen(fd, 128) };

    if rc < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

pub(crate) fn sys_parse_sockaddr(address: &str) -> io::Result<(sockaddr_storage, socklen_t)> {
    let (ip, port) = address
        .rsplit_once(':')
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid address"))?;

    let port: u16 = port
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid port"))?;

    let ip: IpAddr = ip
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid ip"))?;

    let mut storage: sockaddr_storage = unsafe { mem::zeroed() };

    let len = match ip {
        IpAddr::V4(v4) => {
            let addr = unsafe { &mut *(&mut storage as *mut _ as *mut sockaddr_in) };
            addr.sin_family = AF_INET as _;
            addr.sin_port = port.to_be();
            addr.sin_addr.s_addr = u32::from(v4).to_be();
            mem::size_of::<sockaddr_in>()
        }
        IpAddr::V6(v6) => {
            let addr = unsafe { &mut *(&mut storage as *mut _ as *mut sockaddr_in6) };
            addr.sin6_family = AF_INET6 as _;
            addr.sin6_port = port.to_be();
            addr.sin6_addr.s6_addr = v6.octets();
            mem::size_of::<sockaddr_in6>()
        }
    };

    Ok((storage, len as socklen_t))
}

pub(crate) fn sys_shutdown(fd: RawFd, how: Shutdown) -> io::Result<()> {
    let how = match how {
        Shutdown::Read => SHUT_RD,
        Shutdown::Write => SHUT_WR,
        Shutdown::Both => SHUT_RDWR,
    };

    let rc = unsafe { shutdown(fd, how) };

    if rc < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub(crate) fn sys_set_reuseaddr(fd: RawFd) -> io::Result<()> {
    let yes: c_int = 1;
    let rc = unsafe {
        setsockopt(
            fd,
            SOL_SOCKET,
            SO_REUSEADDR,
            &yes as *const _ as *const _,
            mem::size_of::<c_int>() as socklen_t,
        )
    };

    if rc < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub(crate) fn sys_connect(fd: RawFd, addr: &SocketAddr) -> io::Result<()> {
    let (storage, len) = socketaddr_to_storage(addr);

    let rc = unsafe { connect(fd, &storage as *const _ as *const sockaddr, len) };
    if rc < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub(crate) fn socketaddr_to_storage(addr: &SocketAddr) -> (sockaddr_storage, socklen_t) {
    let mut storage: sockaddr_storage = unsafe { mem::zeroed() };

    match addr {
        SocketAddr::V4(v4) => {
            let sa = unsafe { &mut *(&mut storage as *mut _ as *mut sockaddr_in) };
            sa.sin_family = AF_INET as _;
            sa.sin_port = v4.port().to_be();
            sa.sin_addr.s_addr = u32::from(*v4.ip()).to_be();

            (storage, mem::size_of::<sockaddr_in>() as socklen_t)
        }

        SocketAddr::V6(v6) => {
            let sa = unsafe { &mut *(&mut storage as *mut _ as *mut sockaddr_in6) };
            sa.sin6_family = AF_INET6 as _;
            sa.sin6_port = v6.port().to_be();
            sa.sin6_addr.s6_addr = v6.ip().octets();
            sa.sin6_flowinfo = v6.flowinfo();
            sa.sin6_scope_id = v6.scope_id();

            (storage, mem::size_of::<sockaddr_in6>() as socklen_t)
        }
    }
}
