use crate::{Reward, RewardVector, EpisodeRewards};

pub trait RewardConversion {
  fn sum_per_episode(&self) -> RewardVector;
  fn avg_per_episode(&self) -> RewardVector;
  fn median_per_episode(&mut self) -> RewardVector;
}

pub fn avg(list: &[Reward]) -> Reward {
  let sum: f64 = Iterator::sum(list.iter());
  sum / (list.len() as f64)
}

pub fn sum(list: &[Reward]) -> Reward {
  Iterator::sum(list.iter())
}

pub fn min_max(list: &[Reward]) -> (Reward, Reward) {
  let mut max = std::f64::MIN;
  let mut min = std::f64::MAX;
  for reward in list {
    if reward > &max {
      max = *reward;
    }
    if reward < &min {
      min = *reward;
    }
  }
  (min, max)
}

pub fn median(list: &[Reward]) -> Reward {
  let len = list.len();
  let mid = len / 2;
  if len % 2 == 0 {
    avg(&list[(mid - 1)..(mid + 1)])
  } else {
    list[mid]
  }
}

impl RewardConversion for EpisodeRewards {
  fn sum_per_episode(&self) -> RewardVector {
    self.iter().map(|rewards| sum(rewards.as_ref())).collect()
  }
  fn avg_per_episode(&self) -> RewardVector {
    self.iter().map(|rewards| avg(rewards.as_ref())).collect()
  }
  fn median_per_episode(&mut self) -> RewardVector {
    self.sort_by(|a, b| a.partial_cmp(b).unwrap());
    self.iter().map(|rewards| median(rewards.as_ref())).collect()
  }
}