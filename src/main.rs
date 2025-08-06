use std::collections::HashMap;
use std::cmp::max;
use proconio::input;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

const T: u8 = 10;
const DISCOUNT_FACTOR: f64 = 0.985;
const DEBUG: bool = true;

static CALCULATION_COUNT: AtomicUsize = AtomicUsize::new(0);
static MEMO_HITS: AtomicUsize = AtomicUsize::new(0);
static MEMO_MISSES: AtomicUsize = AtomicUsize::new(0);


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct State {
    s: i32,
    h: i32, 
    d: u8, 
    t: u8,
    is_restarted: bool,
}

fn dp(state: State, memo: &mut HashMap<State, (f64, i32)>) -> (f64, i32) {
    let calc_count = CALCULATION_COUNT.fetch_add(1, Ordering::Relaxed);
    
    if let Some(&v) = memo.get(&state) {
        MEMO_HITS.fetch_add(1, Ordering::Relaxed);
        if DEBUG && calc_count % 10000 == 0 {
            println!("[MEMO HIT] Count: {}, State: {:?}, Value: {:.2}", calc_count, state, v.0);
        }
        return v;
    }
    
    MEMO_MISSES.fetch_add(1, Ordering::Relaxed);
    
    if DEBUG && (calc_count < 100 || calc_count % 1000 == 0) {
        println!("[DP START] #{} State: s={}, h={}, d={}, t={}, restart={}", 
                calc_count, state.s, state.h, state.d, state.t, state.is_restarted);
    }

    if is_failed(&state) {
        let result = (0.0, 0);
        if DEBUG && calc_count < 50 {
            println!("[FAILED] State: {:?} -> {:.2}", state, result.0);
        }
        memo.insert(state, result);
        return result;
    }
    
    if state.d >= 5 {
        if state.is_restarted {
            let result = (0.5 * state.s as f64, 0);
            memo.insert(state, result);
            return result;
        }

        let withdrawn_amount = 0.5 * state.s as f64;
        let next_s = state.s / 2;
        if next_s < 100 {
            let result = (0.5 * state.s as f64, 0);
            memo.insert(state, result);
            return result;
        }
        let next_state = State {
            s: next_s,
            h: next_s,
            d: 0,
            t: 0,
            is_restarted: true,
        };
        let (next_value, _) = dp(next_state, memo);
        let result = (withdrawn_amount + next_value, 0);
        memo.insert(state, result);
        return result;
    }
    
    if state.t == T {
        let result = (0.0, 0);
        if DEBUG && calc_count < 50 {
            println!("[TIME UP] State: {:?} -> {:.2}", state, result.0);
        }
        memo.insert(state, result);
        return result;
    }
    
    let max_drawdown_allowed = if state.is_restarted {
        state.h
    } else if state.h <= 2000 {
        2000
    } else {
        state.h
    };
    let current_drawdown = (state.h - state.s).max(0);
    let remaining_drawdown = max_drawdown_allowed - current_drawdown;
    
    if remaining_drawdown < 50 {
        let result = (0.0, 0);
        if DEBUG && calc_count < 100 {
            println!("[NO RISK] State: {:?}, remaining_dd={} -> {:.2}", state, remaining_drawdown, result.0);
        }
        memo.insert(state, result);
        return result;
    }
    
    if DEBUG && calc_count < 100 {
        println!("[RISK CALC] State: {:?}, max_dd={}, current_dd={}, remaining={}", 
                state, max_drawdown_allowed, current_drawdown, remaining_drawdown);
    }

    let mut max_value = f64::MIN;
    let mut optimal_bet = 50;

    let max_bet = std::cmp::min(remaining_drawdown, state.s);

    for w in (50..=4000).step_by(50) {

        if state.is_restarted {
           if w > max_bet {
                break;
           }
        }

        if w > remaining_drawdown {
            break; 
        }

        let p_win = 1.0 / 3.0;
        let p_lose = 2.0 / 3.0;

        let s_win = state.s + w * 2;
        let h_win = max(state.h, s_win);
        let d_win = if w * 2 >= 200 { state.d + 1 } else { state.d };

        let win_state = State {
            s: s_win,
            h: h_win,
            d: d_win,
            t: state.t + 1,
            is_restarted: state.is_restarted,
        };

        let s_lose = state.s - w;
        let lose_state = State {
            s: s_lose,
            h: state.h,
            d: state.d,
            t: state.t + 1,
            is_restarted: state.is_restarted,
        };

        let (win_value, _) = dp(win_state, memo);
        let (lose_value, _) = dp(lose_state, memo);
        let expected = p_win * win_value + p_lose * lose_value * DISCOUNT_FACTOR;
        
        if DEBUG && calc_count < 20 && (w <= 200 || w % 500 == 0) {
            println!("  [BET] w={}, win_val={:.2}, lose_val={:.2}, expected={:.2}", 
                    w, win_value, lose_value, expected);
        }
        
        if expected > max_value {
            max_value = expected;
            optimal_bet = w;
            if DEBUG && calc_count < 50 {
                println!("  [NEW BEST] w={}, value={:.2}", optimal_bet, max_value);
            }
        }
    }

    let result = (max_value, optimal_bet);
    memo.insert(state, result);
    
    if DEBUG && (calc_count < 100 || calc_count % 5000 == 0) {
        println!("[DP END] #{} State: {:?} -> EV={:.2}, Bet=${}", 
                calc_count, state, result.0, result.1);
    }
    
    result
}

fn is_failed(state: &State) -> bool {
    if state.is_restarted {
        state.s <= 0
    } else if state.h <= 2000 {
        state.s < state.h - 2000
    } else {
        state.s <= 0
    }
}

fn main() {
    println!("=== TOPSTEPリスク最適化Solver ===");
    println!();
    
    println!("以下の値を入力してください:");
    println!("口座残高(s) 口座のAllTimeHighの金額(h) 規定額超過日数(d:0-4) ターン数(t:0-9((5-d)+t<10を満たす値))");
    println!("例: 5000 5000 2 3");
    println!();
    
    input! {
        s: i32,
        h: i32, 
        d: u8,
        t: u8,
    }
    
    if d > 4 {
        eprintln!("エラー: d（規定額超過日数）は0-4の範囲で指定してください。入力値: {}", d);
        std::process::exit(1);
    }
    
    if t > 9 {
        eprintln!("エラー: t（ターン数）は0-9の範囲で指定してください。入力値: {}", t);
        std::process::exit(1);
    }
    
    if (5 - d) + t >= 10 {
        eprintln!("エラー: 条件 (5-d)+t<10 を満たしません。現在の値: (5-{})+{}={}", 
                  d, t, (5 - d) + t);
        std::process::exit(1);
    }

    println!();
    println!("=== 計算中... ===");
    
    let start_time = Instant::now();
    
    let init_state = State {
        s: s,
        h: h,
        d: d,
        t: t,
        is_restarted: false,
    };

    if DEBUG {
        println!("[DEBUG] 初期状態: {:?}", init_state);
        println!("[DEBUG] 計算開始...");
    }

    let mut memo = HashMap::new();
    let (value, optimal_bet) = dp(init_state, &mut memo);
    
    let elapsed = start_time.elapsed();
    let total_calcs = CALCULATION_COUNT.load(Ordering::Relaxed);
    let memo_hits = MEMO_HITS.load(Ordering::Relaxed);
    let memo_misses = MEMO_MISSES.load(Ordering::Relaxed);
    
    if DEBUG {
        println!();
        println!("=== デバッグ統計 ===");
        println!("計算時間: {:.3}秒", elapsed.as_secs_f64());
        println!("総計算回数: {}", total_calcs);
        println!("メモ化ヒット: {} ({:.1}%)", memo_hits, memo_hits as f64 / total_calcs as f64 * 100.0);
        println!("メモ化ミス: {} ({:.1}%)", memo_misses, memo_misses as f64 / total_calcs as f64 * 100.0);
        println!("状態数: {}", memo.len());
        println!("計算効率: {:.0} calls/sec", total_calcs as f64 / elapsed.as_secs_f64());
    }

    println!();
    println!("=== 結果 ===");
    println!("口座残高: {}", s);
    println!("MLL: {}", h);  
    println!("規定額超過日数: {}", d);
    println!("ターン数: {}", t);
    println!("EV: {:.2}", value);
    println!("最適ベット額: ${}", optimal_bet);
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_drawdown_calculation_week2() {
        let state = State {
            s: 500,
            h: 500,
            d: 0,
            t: 0,
            is_restarted: true,
        };

        let max_drawdown_allowed = if state.is_restarted {
            state.h
        } else if state.h <= 2000 {
            2000
        } else {
            state.h
        };
        
        let current_drawdown = (state.h - state.s).max(0);
        let remaining_drawdown = max_drawdown_allowed - current_drawdown;

        assert_eq!(max_drawdown_allowed, 500, "2週目でMaxDDは口座残高と同じ500になるべき");
        assert_eq!(current_drawdown, 0, "口座残高とMLLが同じなので現在のドローダウンは0");
        assert_eq!(remaining_drawdown, 500, "最大許容リスクは500になるべき");
    }

    #[test]
    fn test_max_drawdown_with_loss_week2() {
        let state = State {
            s: 300,
            h: 500,
            d: 0,
            t: 0,
            is_restarted: true,
        };

        let max_drawdown_allowed = if state.is_restarted {
            state.h
        } else if state.h <= 2000 {
            2000
        } else {
            state.h
        };
        
        let current_drawdown = (state.h - state.s).max(0);
        let remaining_drawdown = max_drawdown_allowed - current_drawdown;

        assert_eq!(max_drawdown_allowed, 500, "2週目でMaxDDはMLL値500になるべき");
        assert_eq!(current_drawdown, 200, "ドローダウンは500-300=200");
        assert_eq!(remaining_drawdown, 300, "残り許容リスクは500-200=300");
    }

    #[test]
    fn test_max_drawdown_week1_balance_2500() {
        let state = State {
            s: 2500,
            h: 2500,
            d: 0,
            t: 0,
            is_restarted: false,
        };

        let max_drawdown_allowed = if state.is_restarted {
            state.h
        } else if state.h <= 2000 {
            2000
        } else {
            state.h
        };
        
        let current_drawdown = (state.h - state.s).max(0);
        let remaining_drawdown = max_drawdown_allowed - current_drawdown;

        assert_eq!(max_drawdown_allowed, 2500, "1週目でh>2000の場合MaxDDはh=2500");
        assert_eq!(current_drawdown, 0, "残高とMLLが同じなので現在のドローダウンは0");
        assert_eq!(remaining_drawdown, 2500, "最大許容リスクは2500になるべき");
    }

    #[test]
    fn test_max_drawdown_week2_balance_2500() {
        let state = State {
            s: 2500,
            h: 2500,
            d: 0,
            t: 0,
            is_restarted: true,
        };

        let max_drawdown_allowed = if state.is_restarted {
            state.h
        } else if state.h <= 2000 {
            2000
        } else {
            state.h
        };
        
        let current_drawdown = (state.h - state.s).max(0);
        let remaining_drawdown = max_drawdown_allowed - current_drawdown;

        assert_eq!(max_drawdown_allowed, 2500, "2週目でMaxDDはh=2500");
        assert_eq!(current_drawdown, 0, "残高とMLLが同じなので現在のドローダウンは0");
        assert_eq!(remaining_drawdown, 2500, "最大許容リスクは2500になるべき");
    }

    #[test]
    fn test_failure_condition_negative_2500() {
        let state_week1 = State {
            s: 0,
            h: 2500,
            d: 0,
            t: 0,
            is_restarted: false,
        };

        let state_week2 = State {
            s: 0,
            h: 2500,
            d: 0,
            t: 0,
            is_restarted: true,
        };

        assert!(is_failed(&state_week1), "1週目で残高0（実質-2500）は失効");
        assert!(is_failed(&state_week2), "2週目で残高0は失効");
    }
}