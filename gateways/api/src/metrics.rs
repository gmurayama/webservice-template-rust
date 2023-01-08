use prometheus_client::{
    metrics::{counter::Counter, family::Family, histogram::Histogram},
    registry::Registry,
};

#[derive(Clone)]
pub struct ApiMetrics {
    pub request_count: Family<Vec<(String, String)>, Counter>,
    pub request_duration: Family<Vec<(String, String)>, Histogram>,
}

impl ApiMetrics {
    pub fn new(registry: &mut Registry) -> ApiMetrics {
        let request_count = Family::<Vec<(String, String)>, Counter>::default();
        let request_duration =
            Family::<Vec<(String, String)>, Histogram>::new_with_constructor(|| {
                let buckets = [
                    1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0,
                    1250.0, 1500.0, 2000.0,
                ];
                Histogram::new(buckets.into_iter())
            });

        registry.register(
            "request_count",
            "Number of requests received",
            request_count.clone(),
        );

        registry.register(
            "request_duration_ms",
            "Request duration",
            request_duration.clone(),
        );

        ApiMetrics {
            request_count,
            request_duration,
        }
    }
}
