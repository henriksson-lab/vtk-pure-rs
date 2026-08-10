#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum TimerLogEventType {
    Invalid = -1,
    Standalone = 0,
    Start = 1,
    End = 2,
    Inserted = 3,
}

/// VTK origin: `vtkTimerLogEntry`.
#[derive(Debug, Clone)]
pub(crate) struct TimerLogEntry {
    event_type: TimerLogEventType,
    event: String,
    wall_time: f64,
    cpu_ticks: i32,
    indent: i32,
}

impl TimerLogEntry {
    fn event_type(&self) -> TimerLogEventType {
        self.event_type
    }

    fn event(&self) -> &str {
        &self.event
    }

    fn wall_time(&self) -> f64 {
        self.wall_time
    }

    #[cfg(test)]
    fn cpu_ticks(&self) -> i32 {
        self.cpu_ticks
    }

    fn indent(&self) -> i32 {
        self.indent
    }
}

/// Simple timer for measuring elapsed execution time.
///
/// VTK origin: `VTK/Common/System/vtkTimerLog.cxx`.
pub struct TimerLog {
    start_time: f64,
    end_time: f64,
    entries: Vec<TimerLogEntry>,
    max_entries: i32,
    current_indent: i32,
    logging: bool,
    wrap_flag: bool,
}

impl TimerLog {
    /// VTK: `vtkTimerLog::New`.
    pub fn new() -> Self {
        Self {
            start_time: 0.0,
            end_time: 0.0,
            entries: Vec::new(),
            max_entries: 100,
            current_indent: 0,
            logging: true,
            wrap_flag: false,
        }
    }

    /// VTK: `vtkTimerLog::StartTimer`.
    pub fn start_timer(&mut self) {
        self.start_time = Self::get_universal_time();
    }

    /// VTK: `vtkTimerLog::GetElapsedTime`.
    pub fn get_elapsed_time(&self) -> f64 {
        self.end_time - self.start_time
    }

    /// VTK: `vtkTimerLog::StopTimer`.
    pub fn stop_timer(&mut self) {
        self.end_time = Self::get_universal_time();
    }

    /// VTK: `vtkTimerLog::GetUniversalTime`.
    pub fn get_universal_time() -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0)
    }

    /// VTK: `vtkTimerLog::GetCPUTime`.
    pub fn get_cpu_time() -> f64 {
        // Rust's standard library does not expose portable process CPU time.
        Self::get_universal_time()
    }

    /// VTK: `vtkTimerLog::ResetLog`.
    pub fn reset_log(&mut self) {
        self.entries.clear();
        self.current_indent = 0;
        self.wrap_flag = false;
    }

    /// VTK: `vtkTimerLog::CleanupLog`.
    pub fn cleanup_log(&mut self) {
        self.reset_log();
    }

    /// VTK: `vtkTimerLog::SetLogging`.
    pub fn set_logging(&mut self, logging: i32) {
        self.logging = logging != 0;
    }

    /// VTK: `vtkTimerLog::GetLogging`.
    pub fn get_logging(&self) -> i32 {
        i32::from(self.logging)
    }

    /// VTK: `vtkTimerLog::LoggingOn`.
    pub fn logging_on(&mut self) {
        self.set_logging(1);
    }

    /// VTK: `vtkTimerLog::LoggingOff`.
    pub fn logging_off(&mut self) {
        self.set_logging(0);
    }

    /// VTK: `vtkTimerLog::SetMaxEntries`.
    pub fn set_max_entries(&mut self, max_entries: i32) {
        let max_entries = max_entries.max(0);
        self.max_entries = max_entries;
        let max_entries = max_entries as usize;
        if self.entries.len() > max_entries {
            let split = self.entries.len() - max_entries;
            self.entries.drain(..split);
            self.wrap_flag = max_entries > 0;
        } else {
            self.wrap_flag = false;
        }
    }

    /// VTK: `vtkTimerLog::GetMaxEntries`.
    pub fn get_max_entries(&self) -> i32 {
        self.max_entries
    }

    /// VTK: `vtkTimerLog::GetNumberOfEvents`.
    pub fn get_number_of_events(&self) -> i32 {
        self.entries.len() as i32
    }

    /// VTK: `vtkTimerLog::GetEvent`.
    fn event(&self, index: i32) -> Option<&TimerLogEntry> {
        usize::try_from(index)
            .ok()
            .and_then(|index| self.entries.get(index))
    }

    /// VTK: `vtkTimerLog::GetEventString`.
    pub fn get_event_string(&self, index: i32) -> Option<&str> {
        self.event(index).map(TimerLogEntry::event)
    }

    /// VTK: `vtkTimerLog::EventString`.
    #[cfg(test)]
    pub(crate) fn event_type_string(event_type: TimerLogEventType) -> &'static str {
        match event_type {
            TimerLogEventType::Invalid => "Invalid",
            TimerLogEventType::Standalone => "Standalone",
            TimerLogEventType::Start => "Start",
            TimerLogEventType::End => "End",
            TimerLogEventType::Inserted => "Inserted",
        }
    }

    /// VTK: `vtkTimerLog::GetEventType`.
    pub fn get_event_type(&self, index: i32) -> TimerLogEventType {
        self.event(index)
            .map(TimerLogEntry::event_type)
            .unwrap_or(TimerLogEventType::Invalid)
    }

    /// VTK: `vtkTimerLog::GetEventWallTime`.
    pub fn get_event_wall_time(&self, index: i32) -> f64 {
        self.event(index)
            .map(TimerLogEntry::wall_time)
            .unwrap_or(0.0)
    }

    /// VTK: `vtkTimerLog::GetEventIndent`.
    pub fn get_event_indent(&self, index: i32) -> i32 {
        self.event(index).map(TimerLogEntry::indent).unwrap_or(0)
    }

    /// VTK: `vtkTimerLog::InsertTimedEvent`.
    pub fn insert_timed_event(&mut self, event: Option<&str>, time: f64, cpu_ticks: i32) {
        if !self.logging {
            return;
        }
        self.push_entry(TimerLogEntry {
            event_type: TimerLogEventType::Inserted,
            event: event.unwrap_or("").to_string(),
            wall_time: time,
            cpu_ticks,
            indent: self.current_indent,
        });
    }

    /// VTK: `vtkTimerLog::MarkEventInternal`.
    fn mark_event_internal(&mut self, event_type: TimerLogEventType, event: Option<&str>) {
        if !self.logging {
            return;
        }

        self.push_entry(TimerLogEntry {
            event_type,
            event: event.unwrap_or("").to_string(),
            wall_time: Self::get_universal_time(),
            cpu_ticks: 0,
            indent: self.current_indent,
        });
    }

    fn push_entry(&mut self, entry: TimerLogEntry) {
        if self.max_entries <= 0 {
            return;
        }

        let max_entries = self.max_entries as usize;
        if self.entries.len() == max_entries {
            self.entries.remove(0);
            self.wrap_flag = true;
        }
        self.entries.push(entry);
    }

    /// VTK: `vtkTimerLog::MarkEvent`.
    pub fn mark_event(&mut self, event: Option<&str>) {
        self.mark_event_internal(TimerLogEventType::Standalone, event);
    }

    /// VTK: `vtkTimerLog::MarkStartEvent`.
    pub fn mark_start_event(&mut self, event: Option<&str>) {
        self.mark_event_internal(TimerLogEventType::Start, event);
        if self.logging {
            self.current_indent += 1;
        }
    }

    /// VTK: `vtkTimerLog::MarkEndEvent`.
    pub fn mark_end_event(&mut self, event: Option<&str>) {
        self.mark_event_internal(TimerLogEventType::End, event);
        if self.logging {
            self.current_indent -= 1;
        }
    }

    /// VTK: `vtkTimerLog::DumpLog`.
    pub fn dump_log(&self) -> String {
        let mut output = String::new();
        let mut previous_wall_time = 0.0;
        let mut previous_cpu_ticks = 0;
        for (index, entry) in self.entries.iter().enumerate() {
            let delta_wall_time = entry.wall_time - previous_wall_time;
            let delta_cpu_ticks = entry.cpu_ticks - previous_cpu_ticks;
            output.push_str(&format!(
                "{}   {:.6}  {:.6}   {}  {}  {}\n",
                index,
                entry.wall_time,
                delta_wall_time,
                entry.cpu_ticks,
                delta_cpu_ticks,
                entry.event
            ));
            previous_wall_time = entry.wall_time;
            previous_cpu_ticks = entry.cpu_ticks;
        }
        output
    }

    /// VTK: `vtkTimerLog::DumpLogWithIndents`.
    pub fn dump_log_with_indents(&self, threshold: f64) -> String {
        let mut output = String::new();
        for entry in &self.entries {
            if entry.wall_time < threshold {
                continue;
            }
            output.push_str(&"  ".repeat(entry.indent.max(0) as usize));
            output.push_str(&format!("{:.6}: {}\n", entry.wall_time, entry.event));
        }
        output
    }

    /// VTK: `vtkTimerLog::DumpLogWithIndentsAndPercentages`.
    pub fn dump_log_with_indents_and_percentages(&self) -> String {
        self.dump_log_with_indents(0.0)
    }

    /// VTK: `vtkTimerLog::DumpEntry`.
    #[cfg(test)]
    pub(crate) fn dump_entry(&self, index: i32) -> Option<String> {
        self.event(index)
            .map(|e| format!("{:.6} [{}]: {}", e.wall_time, e.cpu_ticks(), e.event))
    }

    /// VTK: `vtkTimerLog::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "TimerLog {{ elapsed_time: {:.6}, number_of_events: {}, max_entries: {} }}",
            self.get_elapsed_time(),
            self.get_number_of_events(),
            self.get_max_entries()
        )
    }
}

/// RAII cleanup guard for a timer log.
///
/// VTK origin: `vtkTimerLogCleanup`.
pub struct TimerLogCleanup<'a> {
    timer_log: Option<&'a mut TimerLog>,
}

impl<'a> TimerLogCleanup<'a> {
    /// VTK: `vtkTimerLogCleanup::vtkTimerLogCleanup`.
    pub fn new(timer_log: &'a mut TimerLog) -> Self {
        Self {
            timer_log: Some(timer_log),
        }
    }

    /// VTK: `vtkTimerLog::CleanupLog`.
    #[cfg(test)]
    pub(crate) fn cleanup_now(mut self) {
        if let Some(timer_log) = self.timer_log.take() {
            timer_log.cleanup_log();
        }
    }
}

impl Drop for TimerLogCleanup<'_> {
    fn drop(&mut self) {
        if let Some(timer_log) = self.timer_log.take() {
            timer_log.cleanup_log();
        }
    }
}

/// RAII start/end event guard for a timer log.
///
/// VTK origin: `vtkTimerLogScope`.
pub struct TimerLogScope<'a> {
    timer_log: Option<&'a mut TimerLog>,
    event: String,
}

impl<'a> TimerLogScope<'a> {
    /// VTK: `vtkTimerLogScope::vtkTimerLogScope`.
    pub fn new(timer_log: &'a mut TimerLog, event: Option<&str>) -> Self {
        let event = event.unwrap_or("").to_string();
        timer_log.mark_start_event(Some(&event));
        Self {
            timer_log: Some(timer_log),
            event,
        }
    }

    /// Finish the scope before drop.
    #[cfg(test)]
    pub(crate) fn end_now(mut self) {
        if let Some(timer_log) = self.timer_log.take() {
            timer_log.mark_end_event(Some(&self.event));
        }
    }
}

impl Drop for TimerLogScope<'_> {
    fn drop(&mut self) {
        if let Some(timer_log) = self.timer_log.take() {
            timer_log.mark_end_event(Some(&self.event));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TimerLog, TimerLogCleanup, TimerLogEventType, TimerLogScope};

    #[test]
    fn mark_event_internal_tracks_event_strings_and_indents() {
        let mut log = TimerLog::new();

        log.mark_start_event(Some("outer"));
        log.mark_event(Some("inner"));
        log.mark_end_event(Some("outer"));

        assert_eq!(log.get_number_of_events(), 3);
        assert_eq!(log.get_event_string(0), Some("outer"));
        assert_eq!(log.get_event_string(1), Some("inner"));
        assert_eq!(log.get_event_string(99), None);
        assert_eq!(log.get_event_indent(0), 0);
        assert_eq!(log.get_event_indent(1), 1);
        assert_eq!(log.get_event_indent(2), 1);
        assert_eq!(
            TimerLog::event_type_string(TimerLogEventType::Start),
            "Start"
        );
    }

    #[test]
    fn cleanup_guard_clears_log_on_drop_or_now() {
        let mut log = TimerLog::new();
        log.mark_event(Some("before"));

        {
            let _cleanup = TimerLogCleanup::new(&mut log);
        }

        assert_eq!(log.get_number_of_events(), 0);

        log.mark_event(Some("again"));
        TimerLogCleanup::new(&mut log).cleanup_now();
        assert_eq!(log.get_number_of_events(), 0);
    }

    #[test]
    fn scope_guard_marks_start_and_end_events() {
        let mut log = TimerLog::new();

        {
            let _scope = TimerLogScope::new(&mut log, Some("work"));
        }

        assert_eq!(log.get_number_of_events(), 2);
        assert_eq!(log.get_event_type(0), TimerLogEventType::Start);
        assert_eq!(log.get_event_type(1), TimerLogEventType::End);
        assert_eq!(log.get_event_string(0), Some("work"));
        assert_eq!(log.get_event_string(1), Some("work"));
        assert_eq!(log.get_event_indent(0), 0);
        assert_eq!(log.get_event_indent(1), 1);
    }
}
