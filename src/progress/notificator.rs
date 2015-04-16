use progress::Timestamp;
use progress::frontier::MutableAntichain;
use progress::count_map::CountMap;

#[derive(Default)]
pub struct Notificator<T: Timestamp> {
    pending:        MutableAntichain<T>,    // requests that have not yet been notified
    frontier:       Vec<MutableAntichain<T>>,    // outstanding work, preventing notification
    available:      CountMap<T>,
    temp:           CountMap<T>,
    changes:        CountMap<T>,
}

impl<T: Timestamp> Notificator<T> {
    pub fn update_frontier_from_cm(&mut self, count_map: &mut [CountMap<T>]) {
        for index in 0..count_map.len() {
            while self.frontier.len() < count_map.len() {
                self.frontier.push(MutableAntichain::new());
            }
            while let Some((time, delta)) = count_map[index].pop() {
                self.frontier[index].update(&time, delta);
            }
            // TODO : If you swap these next three lines in for the above three,
            // TODO : you get an ICE when building against this crate.
            // while let Some((ref time, delta)) = count_map[index].pop() {
            //     self.frontier[index].update(time, delta);
            // }
        }


        // TODO : CRITICAL that we only mark as available the times in the frontier + pending
        // TODO : Not clear where this logic goes (here, on in the iterator; probably there).
        for pend in self.pending.elements.iter() {
            if !self.frontier.iter().any(|x| x.le(pend)) {
                if let Some(val) = self.pending.count(pend) {
                    self.temp.update(pend, -val);
                    self.available.update(pend, val);
                }
            }
        }

        while let Some((pend, val)) = self.temp.pop() {
            self.pending.update(&pend, val);
        }
    }

    pub fn notify_at(&mut self, time: &T) {
        self.changes.update(time, 1);
        self.pending.update(time, 1);

        if !self.frontier.iter().any(|x| x.le(time)) {
            // TODO : Technically you should be permitted to send and notify at the current notification
            // TODO : but this would panic because we have already removed it from the frontier.
            // TODO : A RAII capability for sending/notifying would be good, but the time is almost
            // TODO : exactly that.

            // println!("notificator error? notify_at called with time not le the frontier. {:?}", time);
            // println!("notificator error? {:?} vs {:?}", time, self.frontier);
            // panic!("");
            // self.available.update(time, 1);
        }
    }

    pub fn pull_progress(&mut self, internal: &mut CountMap<T>) {
        while let Some((time, delta)) = self.changes.pop() {
            internal.update(&time, delta);
        }
    }
}

// TODO : This prevents notify_at mid-iteration
impl<T: Timestamp> Iterator for Notificator<T> {
    type Item = (T, i64);
    // TODO : CRITICAL that this method only return the leading edge of times,
    // TODO : and probably good if in so doing it frees up other times.
    fn next(&mut self) -> Option<(T, i64)> {
        if let Some((time, delta)) =  self.available.pop() {
            self.changes.update(&time, -delta);
            Some((time, delta))
        }
        else { None }
    }
}
