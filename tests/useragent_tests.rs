use subconverter_rs::{
    utils::useragent::{match_user_agent, ver_greater_equal},
    SubconverterTarget,
};

#[test]
fn test_ver_greater_equal() {
    assert!(ver_greater_equal("1.2.3", "1.2.3"));
    assert!(ver_greater_equal("1.2.4", "1.2.3"));
    assert!(ver_greater_equal("1.3.0", "1.2.3"));
    assert!(ver_greater_equal("2.0.0", "1.2.3"));
    assert!(ver_greater_equal("1.2.3.4", "1.2.3"));
    assert!(!ver_greater_equal("1.2.2", "1.2.3"));
    assert!(!ver_greater_equal("1.1.9", "1.2.3"));
    assert!(!ver_greater_equal("0.9.9", "1.2.3"));
}

#[test]
fn test_match_user_agent() {
    // Test Clash
    {
        let mut target = SubconverterTarget::Auto;
        let mut clash_new_name = None;
        let mut surge_ver = -1;
        match_user_agent(
            "Clash/1.0.0",
            &mut target,
            &mut clash_new_name,
            &mut surge_ver,
        );
        assert_eq!(target, SubconverterTarget::Clash);
    }

    // Test Clash Premium
    {
        let mut target = SubconverterTarget::Auto;
        let mut clash_new_name = None;
        let mut surge_ver = -1;
        match_user_agent(
            "Clash Premium/1.0.0",
            &mut target,
            &mut clash_new_name,
            &mut surge_ver,
        );
        assert_eq!(target, SubconverterTarget::Clash);
        assert_eq!(clash_new_name, Some(true));
    }

    // Test ShadowRocket
    {
        let mut target = SubconverterTarget::Auto;
        let mut clash_new_name = None;
        let mut surge_ver = -1;
        match_user_agent(
            "Shadowrocket/1.0.0",
            &mut target,
            &mut clash_new_name,
            &mut surge_ver,
        );
        assert_eq!(target, SubconverterTarget::Mixed);
    }

    // Test unknown user agent
    {
        let mut target = SubconverterTarget::Auto;
        let mut clash_new_name = None;
        let mut surge_ver = -1;
        match_user_agent(
            "Unknown Browser",
            &mut target,
            &mut clash_new_name,
            &mut surge_ver,
        );
        assert_eq!(target, SubconverterTarget::Auto);
    }
}
