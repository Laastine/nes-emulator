#[derive(Copy, Clone)]
pub enum PageSize {
  EightKb = 0x2000,
  SixteenKb = 0x4000,
}

impl PageSize {
  pub fn value(&self) -> usize {
    match *self {
      PageSize::EightKb => 0x2000,
      PageSize::SixteenKb => 0x4000,
    }
  }
}

#[derive(Copy, Clone)]
pub enum Page {
  First(PageSize),
  FromNth(usize, PageSize),
  Last(PageSize),
}

pub struct Pager {
  pub data: Vec<u8>,
}

impl Pager {
  pub fn new(data: Vec<u8>) -> Pager {
    Pager {
      data
    }
  }

  pub fn read(&self, page: Page, offset: u16) -> u8 {
    let idx = self.index(page, offset);
    self.data[idx]
  }

  pub fn write(&mut self, page: Page, offset: u16, value: u8) {
    let idx = self.index(page, offset);
    self.data[idx] = value;
  }

  fn page_count(&self, size: PageSize) -> usize {
    if self.data.len() % size.value() != 0 {
      panic!("Page size must divide evenly into data length")
    }

    self.data.len() / (size as usize)
  }

  fn index(&self, page: Page, offset: u16) -> usize {
    match page {
      Page::First(size) => self.index(Page::FromNth(0, size), offset),
      Page::Last(size) => {
        let last_page = self.page_count(size) - 1;
        self.index(Page::FromNth(last_page, size), offset)
      }
      Page::FromNth(nth, size) => {
        if (offset as usize) > (size as usize) {
          panic!("Offset exceeded page size")
        }
        if nth >= self.page_count(size) {
          panic!("Page indexing out bounds")
        }
        nth * (size as usize) + (offset as usize)
      }
    }
  }
}
