#![crate_type = "lib"]

extern crate alloc;
#[cfg(test)] extern crate test;

use std::old_io;
use std::result::Result;
use std::ffi::CString;
use std::old_io::fs::PathExtensions;
//use std::path::posix::Path;
use std::rand::{thread_rng, Rng};

mod unixdatagram;

pub type Priority = u8;

#[allow(non_camel_case_types)]
#[derive(Copy,Clone)]
pub enum Severity {
  LOG_EMERG,
  LOG_ALERT,
  LOG_CRIT,
  LOG_ERR,
  LOG_WARNING,
  LOG_NOTICE,
  LOG_INFO,
  LOG_DEBUG
}

#[allow(non_camel_case_types)]
#[derive(Copy,Clone)]
pub enum Facility {
  LOG_KERN     = 0  << 3,
  LOG_USER     = 1  << 3,
  LOG_MAIL     = 2  << 3,
  LOG_DAEMON   = 3  << 3,
  LOG_AUTH     = 4  << 3,
  LOG_SYSLOG   = 5  << 3,
  LOG_LPR      = 6  << 3,
  LOG_NEWS     = 7  << 3,
  LOG_UUCP     = 8  << 3,
  LOG_CRON     = 9  << 3,
  LOG_AUTHPRIV = 10 << 3,
  LOG_FTP      = 11 << 3,
  LOG_LOCAL0   = 16 << 3,
  LOG_LOCAL1   = 17 << 3,
  LOG_LOCAL2   = 18 << 3,
  LOG_LOCAL3   = 19 << 3,
  LOG_LOCAL4   = 20 << 3,
  LOG_LOCAL5   = 21 << 3,
  LOG_LOCAL6   = 22 << 3,
  LOG_LOCAL7   = 23 << 3
}

pub struct Writer {
  facility: Facility,
  tag:      String,
  hostname: String,
  network:  String,
  raddr:    String,
  client:   String,
  server:   String,
  s:        Box<unixdatagram::UnixDatagram>
}

pub fn init(address: String, facility: Facility, tag: String) -> Result<Box<Writer>, old_io::IoError> {
  let mut path = "/dev/log".to_string();
  if ! Path::new(path.clone()).stat().is_ok() {
    path = "/var/run/syslog".to_string();
    if ! Path::new(path.clone()).stat().is_ok() {
      return Err(old_io::IoError {
        kind: old_io::PathDoesntExist,
        desc: "could not find /dev/log nor /var/run/syslog",
        detail: None
      });
    }
  }
  match tempfile() {
    None => {
      println!("could not generate a tempfile");
      Err(old_io::IoError {
        kind: old_io::PathAlreadyExists,
        desc: "could not generate a temporary file",
        detail: None
      })
    },
    Some(p) => {
      println!("temp file: {}", p);
      unixdatagram::UnixDatagram::bind(&CString::from_slice(p.as_bytes())) .map( |s| {
        Box::new(Writer {
          facility: facility.clone(),
          tag:      tag.clone(),
          hostname: "".to_string(),
          network:  "".to_string(),
          raddr:    address.clone(),
          client:   p.clone(),
          server:   path.clone(),
          s:        Box::new(s)
        })
      })
    }
  }
}

impl Writer {
  pub fn format(&self, severity:Severity, message: String) -> String {
    /*let pid = unsafe { getpid() };
    let f =  format!("<{:u}> {:d} {:s} {:s} {:s} {:d} {:s}",
      self.encode_priority(severity, self.facility),
      1,// version
      time::now_utc().rfc3339(),
      self.hostname, self.tag, pid, message);*/
    // simplified version
    let f =  format!("<{:?}> {:?} {:?}",
      self.encode_priority(severity, self.facility.clone()),
      self.tag, message);
    println!("formatted: {}", f);
    return f;
  }

  fn encode_priority(&self, severity: Severity, facility: Facility) -> Priority {
    return facility as u8 | severity as u8
  }

  pub fn send(&mut self, severity: Severity, message: String) -> Result<(), old_io::IoError> {
    let formatted = self.format(severity, message).into_bytes();
    self.s.sendto(formatted.as_slice(), &CString::from_slice(self.server.as_bytes()))
  }

  pub fn emerg(&mut self, message: String) -> Result<(), old_io::IoError> {
    self.send(Severity::LOG_EMERG, message)
  }

  pub fn alert(&mut self, message: String) -> Result<(), old_io::IoError> {
    self.send(Severity::LOG_ALERT, message)
  }

  pub fn crit(&mut self, message: String) -> Result<(), old_io::IoError> {
    self.send(Severity::LOG_CRIT, message)
  }

  pub fn err(&mut self, message: String) -> Result<(), old_io::IoError> {
    self.send(Severity::LOG_ERR, message)
  }

  pub fn warning(&mut self, message: String) -> Result<(), old_io::IoError> {
    self.send(Severity::LOG_WARNING, message)
  }

  pub fn notice(&mut self, message: String) -> Result<(), old_io::IoError> {
    self.send(Severity::LOG_NOTICE, message)
  }

  pub fn info(&mut self, message: String) -> Result<(), old_io::IoError> {
    self.send(Severity::LOG_INFO, message)
  }

  pub fn debug(&mut self, message: String) -> Result<(), old_io::IoError> {
    self.send(Severity::LOG_DEBUG, message)
  }
}

impl Drop for Writer {
  fn drop(&mut self) {
    let r = old_io::fs::unlink(&Path::new(self.client.clone()));
    if r.is_err() {
      println!("could not delete the client socket: {}", self.client);
    }
  }
}

fn tempfile() -> Option<String> {
  let tmpdir = Path::new("/tmp");
  let mut r = thread_rng();
  for _ in range(0, 1000) {
    let filename: String = r.gen_ascii_chars().take(16).collect();
    let p = tmpdir.join(filename);
    if ! p.stat().is_ok() {
      return p.as_str().map(|s| s.to_string());
    }
  }
  None
}
