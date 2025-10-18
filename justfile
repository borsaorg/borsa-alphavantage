clippy_flags := '-W clippy::all -W clippy::cargo -W clippy::pedantic -W clippy::nursery -A clippy::multiple-crate-versions -D warnings'

test:
  cargo nextest run --all-targets --all-features

lint: 
  cargo clippy --all-targets --all-features --fix --allow-dirty -- {{ clippy_flags }}

fmt:
  cargo fmt --all
