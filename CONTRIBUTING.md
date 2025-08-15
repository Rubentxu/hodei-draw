# Contributing to Hodei Draw

🎉 **Thank you for your interest in contributing to Hodei Draw!** 🎉

We welcome contributions from everyone, whether you're a seasoned Rust developer or just getting started with WebAssembly.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Coding Standards](#coding-standards)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting](#issue-reporting)

## 📜 Code of Conduct

This project adheres to a friendly, inclusive, and respectful environment. Please be kind and considerate in all interactions.

## 🤝 How Can I Contribute?

### 🐛 Bug Reports
- Use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.md)
- Provide clear reproduction steps
- Include browser and OS information
- Add screenshots or GIFs if helpful

### 💡 Feature Requests
- Use the [feature request template](.github/ISSUE_TEMPLATE/feature_request.md)
- Explain the use case and expected behavior
- Consider if it aligns with the project roadmap

### 🛠️ Code Contributions
- Pick an issue labeled `good first issue` to get started
- Fork the repository and create a feature branch
- Write clean, tested code following our standards
- Submit a pull request with a clear description

## 🚀 Development Setup

### Prerequisites

```bash
# Install Rust (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add wasm32 target
rustup target add wasm32-unknown-unknown

# Install Trunk (WASM build tool)
cargo install trunk

# Install Node.js (for CSS processing)
# Download from https://nodejs.org/ (18+ required)
```

### Local Development

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/hodei-draw.git
cd hodei-draw

# Install dependencies
cd apps/app-web && npm install && cd ../..

# Start development server
make serve
# OR
cd apps/app-web && trunk serve --open
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p momentum-core

# Run with coverage
cargo tarpaulin --out html
```

## 🎯 Coding Standards

### Rust Code Style

```bash
# Format code (required before commit)
cargo fmt

# Run clippy (required before commit)
cargo clippy -- -D warnings

# Check all features
cargo clippy --all-features -- -D warnings
```

### Code Organization

- **Domain Logic**: Place in `crates/core/`
- **ECS Components**: Place in `crates/ecs/`
- **UI Components**: Place in `crates/ui-leptos/`
- **Design System**: Place in `crates/design-system/`

### Documentation

```rust
/// Brief description of what this function does
///
/// # Arguments
///
/// * `param` - Description of parameter
///
/// # Returns
///
/// Description of return value
///
/// # Examples
///
/// ```
/// let result = my_function(42);
/// assert_eq!(result, expected);
/// ```
pub fn my_function(param: i32) -> String {
    // Implementation
}
```

## 📝 Commit Guidelines

We follow [Conventional Commits](https://conventionalcommits.org/) specification:

```
type(scope): brief description

[optional body]

[optional footer]
```

### Types
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### Scopes
- `core`: Core domain logic
- `ecs`: Entity Component System
- `ui`: User interface
- `design`: Design system
- `app`: Main application
- `build`: Build system
- `ci`: Continuous integration

### Examples

```bash
git commit -m "feat(ui): add shape selection toolbar"
git commit -m "fix(core): resolve rendering race condition"
git commit -m "docs(readme): update installation instructions"
```

## 🔄 Pull Request Process

### Before Submitting

1. **Create a Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Write Tests**
   - Add unit tests for new functionality
   - Ensure existing tests pass
   - Add integration tests if needed

3. **Code Quality Checks**
   ```bash
   # All must pass
   cargo fmt --check
   cargo clippy -- -D warnings  
   cargo test
   ```

4. **Update Documentation**
   - Update relevant documentation
   - Add doc comments for public APIs
   - Update README if needed

### Submitting the PR

1. **Push to Your Fork**
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create Pull Request**
   - Use the provided PR template
   - Reference related issues
   - Include screenshots/GIFs for UI changes
   - Describe testing done

3. **PR Review Process**
   - Automated checks must pass
   - At least one maintainer review required
   - Address feedback promptly
   - Maintain clean commit history

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Manual testing completed
- [ ] All tests pass

## Checklist
- [ ] Code formatted (`cargo fmt`)
- [ ] Clippy warnings addressed
- [ ] Documentation updated
- [ ] Self-reviewed code
```

## 🐛 Issue Reporting

### Bug Reports Should Include:

- **Clear Title**: Concise description of the issue
- **Steps to Reproduce**: Numbered list of actions
- **Expected Behavior**: What should happen
- **Actual Behavior**: What actually happens
- **Environment**: Browser, OS, version info
- **Screenshots**: If visual issue
- **Console Logs**: Any error messages

### Feature Requests Should Include:

- **Problem Statement**: What problem does this solve?
- **Proposed Solution**: How should it work?
- **Alternatives**: Other approaches considered
- **Use Cases**: Who would benefit and how?

## 🏗️ Architecture Guidelines

### Hexagonal Architecture

We follow hexagonal architecture principles:

```
crates/
├── core/          # Domain logic (no dependencies)
├── ecs/           # State management
├── ui-leptos/     # UI framework adapter  
└── design-system/ # UI components
```

### Dependency Rules

- **Core**: No external dependencies on UI or infrastructure
- **ECS**: Can depend on Core
- **UI**: Can depend on Core and ECS
- **Design System**: Independent UI components

## 🎨 Design System Guidelines

### Component Development

```rust
#[component]
pub fn MyComponent(
    #[prop(into)] title: String,
    #[prop(optional)] variant: Option<ButtonVariant>,
    children: Children,
) -> impl IntoView {
    // Implementation
}
```

### Styling Conventions

- Use Tailwind CSS utility classes
- Follow BEM naming for custom CSS
- Maintain dark/light theme compatibility
- Use CSS custom properties for theming

## 🔧 Troubleshooting

### Common Issues

**WASM Build Fails**
```bash
# Clean and rebuild
cargo clean
trunk clean
trunk build
```

**CSS Not Updating**
```bash
# Rebuild CSS
cd apps/app-web
npm run build-css
```

**Port Already in Use**
```bash
# Use different port
trunk serve --port 8081
```

## 🤔 Questions?

- **Discord**: [Join our community](https://discord.gg/hodei-draw) (coming soon)
- **GitHub Discussions**: [Ask questions](https://github.com/Rubentxu/hodei-draw/discussions)
- **Issues**: [Report bugs or request features](https://github.com/Rubentxu/hodei-draw/issues)

---

**Thank you for contributing to Hodei Draw!** 🎨✨