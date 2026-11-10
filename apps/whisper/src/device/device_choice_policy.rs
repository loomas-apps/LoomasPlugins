/// Where the speech model runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceChoice {
    /// What Loomas grants, else the graphics card when this build has one.
    Auto,
    Cpu,
    /// The graphics card with this number (0 when none is named).
    Gpu(i32),
}

/// What to run on, and why a graphics card asked for is not used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevicePlan {
    /// The graphics card to use, or `None` for the processor.
    pub gpu: Option<i32>,
    /// Why the processor is used when a card was asked for or granted.
    pub fallback: Option<String>,
}

impl DeviceChoice {
    /// Reads `auto`, `cpu`, `gpu` or `gpu:N`, in any case; `None` for anything else.
    pub fn parse(text: &str) -> Option<DeviceChoice> {
        match text.trim().to_lowercase().as_str() {
            "auto" => Some(DeviceChoice::Auto),
            "cpu" => Some(DeviceChoice::Cpu),
            "gpu" => Some(DeviceChoice::Gpu(0)),
            other => other
                .strip_prefix("gpu:")
                .and_then(|number| number.parse().ok())
                .map(DeviceChoice::Gpu),
        }
    }
}

/// Decides where to run: a caller that left it to `auto` takes what the host `granted` (the word
/// it offered in `hello`), and a caller that named a device keeps it. Without a grant, `auto` uses
/// the first card. A build without graphics card support (`has_gpu` false) always runs on the
/// processor, and says so when a card was wanted.
pub fn plan_device(asked: DeviceChoice, granted: Option<&str>, has_gpu: bool) -> DevicePlan {
    let wanted = match (asked, granted.and_then(DeviceChoice::parse)) {
        (DeviceChoice::Auto, Some(grant)) => grant,
        (DeviceChoice::Auto, None) => DeviceChoice::Gpu(0),
        (chosen, _) => chosen,
    };
    let explicit = asked != DeviceChoice::Auto || granted.is_some();
    match wanted {
        DeviceChoice::Gpu(number) if has_gpu => DevicePlan {
            gpu: Some(number),
            fallback: None,
        },
        DeviceChoice::Gpu(_) => DevicePlan {
            gpu: None,
            fallback: explicit.then(|| {
                "this build of the plugin has no graphics card support (Vulkan)".to_string()
            }),
        },
        _ => DevicePlan {
            gpu: None,
            fallback: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_takes_the_grant_and_a_named_device_stays() {
        assert_eq!(
            plan_device(DeviceChoice::Auto, Some("gpu:1"), true).gpu,
            Some(1)
        );
        assert_eq!(plan_device(DeviceChoice::Auto, Some("cpu"), true).gpu, None);
        assert_eq!(plan_device(DeviceChoice::Auto, None, true).gpu, Some(0));
        assert_eq!(plan_device(DeviceChoice::Cpu, Some("gpu"), true).gpu, None);
        assert_eq!(DeviceChoice::parse("GPU:2"), Some(DeviceChoice::Gpu(2)));
        assert_eq!(DeviceChoice::parse("vulkan"), None);
    }

    #[test]
    fn a_build_without_a_card_runs_on_the_processor_and_says_why_only_when_one_was_wanted() {
        let plan = plan_device(DeviceChoice::Gpu(0), None, false);
        assert_eq!(plan.gpu, None);
        assert!(plan.fallback.unwrap().contains("Vulkan"));
        assert_eq!(plan_device(DeviceChoice::Auto, None, false).fallback, None);
    }
}
