use borsa_alphavantage::AvConnector;
use borsa_core::BorsaConnector;
use borsa_core::{
    Currency, Earnings, EarningsQuarter, EarningsQuarterEps, EarningsYear, Money, Period,
};

#[test]
fn earnings_conversion_structure() {
    // Test that the earnings conversion produces the expected structure
    // We can't easily test the full conversion without network calls,
    // but we can test that the connector is properly set up for earnings

    let av = AvConnector::new_with_key("DUMMY");

    // Test that the connector supports earnings via provider accessor
    assert!(av.as_earnings_provider().is_some());
}

#[test]
fn earnings_data_structure_validation() {
    // Test that the expected earnings data structure is correct
    let yearly = vec![EarningsYear {
        year: 2023,
        revenue: None,  // Alpha Vantage doesn't provide revenue in earnings endpoint
        earnings: None, // Alpha Vantage doesn't provide earnings in earnings endpoint
    }];

    let quarterly = vec![EarningsQuarter {
        period: "2024Q1".parse::<Period>().unwrap(),
        revenue: None,  // Alpha Vantage doesn't provide revenue in earnings endpoint
        earnings: None, // Alpha Vantage doesn't provide earnings in earnings endpoint
    }];

    let quarterly_eps = vec![EarningsQuarterEps {
        period: "2024Q1".parse::<Period>().unwrap(),
        actual: Some(
            Money::from_canonical_str("2.99", Currency::Iso(borsa_core::IsoCurrency::USD)).unwrap(),
        ),
        estimate: Some(
            Money::from_canonical_str("2.70", Currency::Iso(borsa_core::IsoCurrency::USD)).unwrap(),
        ),
    }];

    let earnings = Earnings {
        yearly,
        quarterly,
        quarterly_eps,
    };

    // Validate the structure
    assert_eq!(earnings.yearly.len(), 1);
    assert_eq!(earnings.quarterly.len(), 1);
    assert_eq!(earnings.quarterly_eps.len(), 1);

    assert_eq!(earnings.yearly[0].year, 2023);
    assert!(matches!(
        earnings.quarterly[0].period,
        Period::Quarter { .. } | Period::Year { .. } | Period::Date(_)
    ));
    assert_eq!(
        earnings.quarterly_eps[0]
            .actual
            .as_ref()
            .unwrap()
            .amount()
            .to_string(),
        "2.99"
    );
    assert_eq!(
        earnings.quarterly_eps[0]
            .estimate
            .as_ref()
            .unwrap()
            .amount()
            .to_string(),
        "2.70"
    );
}

#[test]
fn convert_earnings_periods_parse() {
    let _y = EarningsQuarterEps {
        period: "2024Q1".parse::<Period>().unwrap(),
        actual: None,
        estimate: None,
    };
    let q = EarningsQuarter {
        period: "2024Q1".parse::<Period>().unwrap(),
        revenue: None,
        earnings: None,
    };
    assert!(matches!(
        q.period,
        Period::Quarter { .. } | Period::Year { .. } | Period::Date(_)
    ));
}
