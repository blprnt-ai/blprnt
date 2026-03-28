#[derive(Clone, Debug)]
pub struct FileSandboxFlags {
  pub yolo:      bool,
  pub read_only: bool,
}

#[derive(Clone, Debug)]
pub struct SandboxFlags {
  pub file:           FileSandboxFlags,
  pub network_access: bool,
}

impl SandboxFlags {
  pub fn is_yolo(&self) -> bool {
    self.file.yolo
  }

  pub fn is_read_only(&self) -> bool {
    self.file.read_only
  }

  pub fn is_network_access(&self) -> bool {
    self.network_access
  }
}

impl Default for SandboxFlags {
  fn default() -> Self {
    Self { file: FileSandboxFlags { yolo: false, read_only: false }, network_access: false }
  }
}
