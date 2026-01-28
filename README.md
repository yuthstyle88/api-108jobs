<div align="center">

# 108Jobs

**A Freelance Job Marketplace Platform for Thailand**

[![GitHub](https://img.shields.io/github/license/yuthstyle88/api-108jobs)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.81%2B-orange.svg)](https://www.rust-lang.org)
[![Build](https://img.shields.io/badge/build-passing-brightgreen)]()

</div>

## About 108Jobs

108Jobs is a comprehensive freelance job marketplace platform built for the Thai market. It connects employers with talented freelancers, offering a complete solution for posting jobs, finding work, managing projects, and handling payments securely.

### Key Features

**For Employers:**
- Post job listings and find qualified freelancers
- Review freelancer profiles and ratings
- Manage projects through workflow stages
- Secure payment escrow system
- Real-time chat with freelancers
- Track delivery progress (for delivery jobs)

**For Freelancers:**
- Browse and apply to job opportunities
- Build a professional profile with portfolio
- Receive payment securely through the platform
- Chat with employers
- Track work progress and submit deliverables
- Get rated and build reputation

**For Delivery Riders:**
- Real-time location tracking
- Delivery job management
- Mobile-friendly interface

### Built With

- **Backend**: Rust, Actix-web, Diesel ORM
- **Database**: PostgreSQL
- **Real-time**: WebSocket support for live updates
- **Email**: Built-in email notifications with multi-language support (Thai, English, Vietnamese)
- **Payments**: Integration with SCB (Siam Commercial Bank)

## Project Structure

```
.
├── crates/
│   ├── email/          # Email notification module
│   ├── api/            # API endpoints
│   ├── db_schema/      # Database models and schema
│   ├── db_views/       # Database views for common queries
│   ├── utils/          # Shared utilities and error handling
│   ├── routes/         # HTTP route handlers
│   ├── ws/             # WebSocket support
│   └── workflow/       # Business logic for job workflows
├── migrations/         # Database migrations
└── src/               # Main application entry point
```

## Installation

### Prerequisites

- Rust 1.81 or later
- PostgreSQL 14 or later
- Redis (for caching and pub/sub)

### Development Setup

1. Clone the repository:
```bash
git clone https://github.com/yuthstyle88/api-108jobs.git
cd api-108jobs
```

2. Set up environment variables:
```bash
cp config/default.hoi config/config.hoi
# Edit config/config.hoi with your database and email settings
```

3. Run database migrations:
```bash
cargo run --release -- migration run
```

4. Start the server:
```bash
cargo run --release
```

The API server will start on `http://localhost:8080` by default.

## Email Module

The `crates/email` crate is a standalone email library that can be used independently:

```toml
[dependencies]
app_108jobs_email = { path = "./crates/email" }
```

```rust
use app_108jobs_email::{account, admin, notifications};

// Send password reset email
account::send_password_reset_email(&user, &mut pool, &settings).await?;

// Send application notification to admins
admin::send_new_applicant_email_to_admins("username", &mut pool, &settings).await?;

// Send notification
notifications::send_mention_email(&user, content, &person, link, &settings).await;
```

## API Documentation

API documentation is available at `/docs` endpoint when running the server (if enabled).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the AGPL-3.0 License - see the [LICENSE](LICENSE) file for details.

## Support

For support, please email support@108jobs.com or open an issue in this repository.

## Credits

- **Original Project**: Based on Lemmy (fediverse link aggregator)
- **Adapted for**: 108Jobs - Thai Freelance Marketplace
- **Repository**: https://github.com/yuthstyle88/api-108jobs
- **Website**: https://108jobs.com
