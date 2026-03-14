#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoolValueSource {
  Init,
  Engine,
  Policy,
}

#[derive(Clone, Debug)]
pub struct BoolValue {
  value:  bool,
  set_by: BoolValueSource,
}

impl BoolValue {
  pub fn from_source(value: bool, source: BoolValueSource) -> Self {
    Self { value, set_by: source }
  }
}

#[derive(Clone, Debug)]
pub struct FileSandboxFlags {
  yolo:      BoolValue,
  read_only: BoolValue,
}

#[derive(Clone, Debug)]
pub struct SandboxFlags {
  file:           FileSandboxFlags,
  network_access: BoolValue,
}

impl SandboxFlags {
  pub fn set_yolo(&mut self, value: bool, source: BoolValueSource) {
    if self.file.yolo.set_by == BoolValueSource::Init {
      self.file.yolo = BoolValue::from_source(value, source);
    }
  }

  pub fn is_yolo(&self) -> bool {
    self.file.yolo.value
  }

  pub fn set_read_only(&mut self, value: bool, source: BoolValueSource) {
    if self.file.read_only.set_by == BoolValueSource::Init {
      self.file.read_only = BoolValue::from_source(value, source);
    }
  }

  pub fn is_read_only(&self) -> bool {
    self.file.read_only.value
  }

  pub fn set_network_access(&mut self, value: bool, source: BoolValueSource) {
    if self.network_access.set_by == BoolValueSource::Init {
      self.network_access = BoolValue::from_source(value, source);
    }
  }

  pub fn is_network_access(&self) -> bool {
    self.network_access.value
  }

  pub fn pretty_print_file_access(&self) -> String {
    let mut output = String::new();

    output.push('\n');
    output.push_str("  yolo (no restrictions): ");
    output.push_str(&self.is_yolo().to_string());
    output.push('\n');
    output.push_str("  read_only (only read files): ");
    output.push_str(&self.is_read_only().to_string());
    output.push('\n');

    output
  }

  pub fn pretty_print(&self) -> String {
    let mut output = String::new();

    output.push('\n');
    output.push_str("  yolo (no restrictions): ");
    output.push_str(&self.is_yolo().to_string());
    output.push('\n');
    output.push_str("  read_only (only read files): ");
    output.push_str(&self.is_read_only().to_string());
    output.push('\n');
    output.push_str("  network_access (allow network access): ");
    output.push_str(&self.is_network_access().to_string());
    output.push('\n');

    output
  }
}

impl Default for SandboxFlags {
  fn default() -> Self {
    Self {
      file:           FileSandboxFlags {
        yolo:      BoolValue { value: false, set_by: BoolValueSource::Init },
        read_only: BoolValue { value: false, set_by: BoolValueSource::Init },
      },
      network_access: BoolValue { value: true, set_by: BoolValueSource::Init },
    }
  }
}
