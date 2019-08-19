mkdir deployment
cp target/release/scheduler target/release/podrocket target/release/podcast-cli deployment/
strip deployment/scheduler deployment/podrocket deployment/podcast-cli
