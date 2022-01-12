//! Calendar events (iCal `VEVENT` items)

use std::{collections::HashMap, fmt::Display};

use chrono::{DateTime, Utc, Weekday};
use ical::property::Property;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{item::SyncStatus, utils::random_url};

pub const RRULE_FIELD_FREQ: &str = "FREQ";
pub const RRULE_VALUE_YEARLY: &str = "YEARLY";
pub const RRULE_VALUE_MONTHLY: &str = "MONTHLY";
pub const RRULE_VALUE_WEEKLY: &str = "WEEKLY";
pub const RRULE_VALUE_DAILY: &str = "DAILY";
pub const RRULE_VALUE_HOURLY: &str = "HOURLY";

pub const RRULE_FIELD_BYMONTH: &str = "BYMONTH";

pub const RRULE_FIELD_BYMONTHDAY: &str = "BYMONTHDAY";

pub const RRULE_FIELD_BYDAY: &str = "BYDAY";
pub const RRULE_VALUE_BYDAY_MONDAY: &str = "MO";
pub const RRULE_VALUE_BYDAY_TUESDAY: &str = "TU";
pub const RRULE_VALUE_BYDAY_WEDNESDAY: &str = "WE";
pub const RRULE_VALUE_BYDAY_THURSDAY: &str = "TH";
pub const RRULE_VALUE_BYDAY_FRIDAY: &str = "FR";
pub const RRULE_VALUE_BYDAY_SATURDAY: &str = "SA";
pub const RRULE_VALUE_BYDAY_SUNDAY: &str = "SU";

pub const RRULE_FIELD_BYSETPOS: &str = "BYSETPOS";
pub const RRULE_VALUE_BYSETPOS_FIRST: &str = "1";
pub const RRULE_VALUE_BYSETPOS_SECOND: &str = "2";
pub const RRULE_VALUE_BYSETPOS_THIRD: &str = "3";
pub const RRULE_VALUE_BYSETPOS_FOURTH: &str = "4";
pub const RRULE_VALUE_BYSETPOS_LAST: &str = "-1";

pub const RRULE_FIELD_INTERVAL: &str = "INTERVAL";

pub const RRULE_FIELD_COUNT: &str = "COUNT";
pub const RRULE_FIELD_UNTIL: &str = "UNTIL";

/// A calendar event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    /// Persistent, globally unique identifier for the calendar component
    /// The [RFC](https://tools.ietf.org/html/rfc5545#page-117) recommends concatenating a timestamp with the server's domain name.
    /// UUID are even better so we'll generate them, but we have to support tasks from the server, that may have any arbitrary strings here.
    pub(crate) uid: String,
    /// The task URL
    pub(crate) url: Url,
    pub(crate) ical_prod_id: String,
    /// The sync status of this item
    pub(crate) sync_status: SyncStatus,
    /// The last time this item was modified
    pub(crate) last_modified: DateTime<Utc>,
    /// The time this item was created.
    /// This is not required by RFC5545. This will be populated in events created by this crate, but can be None for tasks coming from a server
    pub(crate) creation_date: Option<DateTime<Utc>>,

    /// The event name
    pub(crate) name: String,
    /// Whether the event is defined for full days or not.
    /// `start` and `end` must be interpreted as Date instead of DateTime if this field is true.
    pub(crate) full_day: bool,
    /// Start date/time of the event
    pub(crate) start: DateTime<Utc>,
    /// End date/time of the event
    pub(crate) end: DateTime<Utc>,
    /// Location of the event
    pub(crate) location: Option<String>,
    /// Repetition of the event.
    /// See https://www.kanzaki.com/docs/ical/rrule.html
    pub(crate) repeat: Option<Vec<(String, String)>>,
    /// Notes/Description of the event
    pub(crate) description: Option<String>,

    pub(crate) extra_parameters: Vec<Property>,
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
            && self.url == other.url
            && self.ical_prod_id == other.ical_prod_id
            && self.sync_status == other.sync_status
            && self.last_modified == other.last_modified
            && self.creation_date == other.creation_date
            && self.name == other.name
            && self.full_day == other.full_day
            && self.start == other.start
            && self.end == other.end
            && self.location == other.location
            && self.repeat == other.repeat
            && self.description == other.description
    }
}

impl Event {
    pub fn new(
        uid: String,
        parent_calendar_url: &Url,
        name: String,
        full_day: bool,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        sync_status: SyncStatus,
    ) -> Self {
        let url = parent_calendar_url
            .join(&format!("{}.ics", uid))
            .unwrap_or(random_url(parent_calendar_url));
        Self {
            uid,
            url,
            sync_status,
            ical_prod_id: crate::ical::default_prod_id(),
            creation_date: Some(Utc::now()),
            last_modified: Utc::now(),
            name,
            full_day,
            start,
            end,
            location: None,
            repeat: None,
            description: None,
            extra_parameters: Vec::new(),
        }
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn uid(&self) -> &str {
        &self.uid
    }

    pub fn ical_prod_id(&self) -> &str {
        &self.ical_prod_id
    }

    pub fn creation_date(&self) -> Option<&DateTime<Utc>> {
        self.creation_date.as_ref()
    }

    pub fn last_modified(&self) -> &DateTime<Utc> {
        &self.last_modified
    }

    pub fn sync_status(&self) -> &SyncStatus {
        &self.sync_status
    }

    pub fn set_sync_status(&mut self, new_status: SyncStatus) {
        self.sync_status = new_status;
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether the event is defined for full days or not.
    /// `start` and `end` must be interpreted as Date instead of DateTime if this field is true.
    pub fn full_day(&self) -> bool {
        self.full_day
    }

    pub fn start(&self) -> &DateTime<Utc> {
        &self.start
    }

    pub fn end(&self) -> &DateTime<Utc> {
        &self.end
    }

    pub fn location(&self) -> Option<&String> {
        self.location.as_ref()
    }

    pub fn set_location(&mut self, location: String) {
        self.location = Some(location)
    }

    /// The repetition of the event.
    /// See https://www.kanzaki.com/docs/ical/rrule.html
    pub fn repeat(&self) -> Option<&Vec<(String, String)>> {
        self.repeat.as_ref()
    }
    pub(crate) fn repeat_string(&self) -> Option<String> {
        self.repeat
            .as_ref()
            .map(|r| r.iter().map(|(k, v)| format!("{}={}", k, v)).join(";"))
    }
    /// The repetition of the event.
    /// See https://www.kanzaki.com/docs/ical/rrule.html
    pub fn set_repeat(&mut self, repeat: Vec<(String, String)>) {
        self.repeat = Some(repeat)
    }

    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    pub fn set_description(&mut self, description: String) {
        self.description = Some(description)
    }

    /// All parameters that are not parsed as fields of the event struct.
    pub fn extra_parameters(&self) -> &Vec<Property> {
        &self.extra_parameters
    }
}
