use std::{collections::HashSet, path::PathBuf};

use yazi_dds::Pubsub;
use yazi_macro::render;
use yazi_shared::{Id, event::{Cmd, Data}, fs::{Url, Urn}};

use crate::manager::Manager;

struct Opt {
	url: Option<Url>,
	tab: Option<Id>,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self { url: c.take_first().and_then(Data::into_url), tab: c.get("tab").and_then(Data::as_id) }
	}
}
impl From<Option<Url>> for Opt {
	fn from(url: Option<Url>) -> Self { Self { url, tab: None } }
}

impl Manager {
	#[yazi_codegen::command]
	pub fn hover(&mut self, opt: Opt) {
		if let Some(u) = opt.url {
			self.hover_do(u, opt.tab);
		} else {
			self.current_or_mut(opt.tab).repos(None);
		}

		// Repeek
		self.peek(false);

		// Refresh watcher
		let mut to_watch = HashSet::with_capacity(3 * self.tabs.len());
		for tab in self.tabs.iter() {
			to_watch.insert(tab.cwd());
			if let Some(ref p) = tab.parent {
				to_watch.insert(&p.url);
			}
			if let Some(h) = tab.hovered().filter(|&h| h.is_dir()) {
				to_watch.insert(&h.url);
			}
		}
		self.watcher.watch(to_watch);

		// Publish through DDS
		Pubsub::pub_from_hover(self.active().id, self.hovered().map(|h| &h.url));
	}

	fn hover_do(&mut self, url: Url, tab: Option<Id>) {
		// Hover on the file
		if let Ok(p) = url.strip_prefix(&self.current_or(tab).url).map(PathBuf::from) {
			render!(self.current_or_mut(tab).repos(Some(Urn::new(&p))));
		}

		// Turn on tracing
		if self.current_or(tab).hovered().is_some_and(|f| url == f.url) {
			// `hover(Some)` occurs after user actions, such as create, rename, reveal, etc.
			// At this point, it's intuitive to track the location of this file regardless.
			self.current_or_mut(tab).tracing = true;
		}
	}
}
