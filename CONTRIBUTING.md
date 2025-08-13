# 🤝 **Contributing to AppTest_v05**

Thank you for your interest in contributing to **AppTest_v05**! This document provides guidelines and information for contributors.

## 📋 **Table of Contents**

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Code Standards](#code-standards)
- [Testing Guidelines](#testing-guidelines)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting](#issue-reporting)
- [Feature Requests](#feature-requests)
- [Documentation](#documentation)
- [Community](#community)

## 📜 **Code of Conduct**

This project adheres to the **Contributor Covenant Code of Conduct**. By participating, you are expected to uphold this code.

### **Our Standards**

- **Respectful Communication** - Be respectful and inclusive
- **Professional Behavior** - Maintain professional standards
- **Constructive Feedback** - Provide helpful, constructive feedback
- **Inclusive Environment** - Welcome contributors from all backgrounds

### **Unacceptable Behavior**

- **Harassment** - Any form of harassment or discrimination
- **Trolling** - Deliberate disruption or inflammatory behavior
- **Spam** - Unwanted promotional content or spam
- **Inappropriate Content** - Offensive or inappropriate material

## 🚀 **Getting Started**

### **Prerequisites**

Before contributing, ensure you have:

- **Rust 1.70+** installed and configured
- **Node.js 18+** and **Bun** for frontend development
- **Git** for version control
- **Basic knowledge** of Rust, TypeScript, and IoT concepts

### **Fork and Clone**

1. **Fork** the repository on GitHub
2. **Clone** your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/AppTest_v05.git
   cd AppTest_v05
   ```
3. **Add upstream** remote:
   ```bash
   git remote add upstream https://github.com/yashrajzala/AppTest_v05.git
   ```

### **Branch Strategy**

- **main**: Production-ready code
- **develop**: Development and integration branch
- **feature/***: New features and enhancements
- **bugfix/***: Bug fixes and patches
- **hotfix/***: Critical production fixes

## 🔧 **Development Setup**

### **Backend Setup (Rust)**

```bash
cd src-tauri

# Install Rust dependencies
cargo check

# Run tests
cargo test

# Check code quality
cargo clippy
cargo fmt

# Build for development
cargo build

# Run in development mode
cargo run
```

### **Frontend Setup (SvelteKit)**

```bash
# Install dependencies
bun install

# Type checking
bun run check

# Development server
bun run dev

# Build for production
bun run build
```

### **Full Application**

```bash
# Start Tauri development mode
bun run tauri dev

# Build production application
bun run tauri build
```

## 📝 **Code Standards**

### **Rust Standards**

#### **Code Style**
- Use **rustfmt** for consistent formatting
- Follow **clippy** recommendations
- Use **4 spaces** for indentation
- Maximum **100 characters** per line

#### **Naming Conventions**
```rust
// Functions and variables: snake_case
pub fn process_sensor_data() -> Result<Data, Error> {
    let sensor_value = 42.0;
    // ...
}

// Constants: SCREAMING_SNAKE_CASE
const MAX_SAMPLES_PER_NODE: usize = 64;

// Types: PascalCase
pub struct SensorReading {
    pub timestamp: Instant,
    pub value: f32,
}
```

#### **Error Handling**
```rust
// Use Result<T, E> for fallible operations
pub fn decode_payload(data: &[u8]) -> Result<Decoded, DecodeError> {
    if data.len() < MIN_PAYLOAD_SIZE {
        return Err(DecodeError::InsufficientData);
    }
    // ... decoding logic
}

// Use Option<T> for optional values
pub fn get_sensor_value(id: u16) -> Option<f32> {
    // ... lookup logic
}
```

#### **Documentation**
```rust
/// Decodes binary sensor payload into structured data.
///
/// # Arguments
/// * `data` - Raw binary payload from ESP32 node
///
/// # Returns
/// * `Some(Decoded)` - Successfully decoded sensor data
/// * `None` - Invalid or malformed payload
///
/// # Examples
/// ```
/// let payload = [0x01, 0x00, 0x02, 0x00, ...];
/// if let Some(decoded) = decode_payload(&payload) {
///     println!("Temperature: {}°C", decoded.air_temp_c);
/// }
/// ```
pub fn decode_payload(data: &[u8]) -> Option<Decoded> {
    // ... implementation
}
```

### **TypeScript Standards**

#### **Code Style**
- Use **Prettier** for formatting
- Follow **ESLint** rules
- Use **2 spaces** for indentation
- Maximum **80 characters** per line

#### **Type Safety**
```typescript
// Use strict typing
interface SensorData {
  timestamp: number;
  temperature: number;
  humidity: number;
}

// Avoid any type
function processData(data: SensorData): ProcessedData {
  // ... processing logic
}

// Use union types for variants
type SensorType = 'temperature' | 'humidity' | 'pressure';
```

#### **Component Structure**
```typescript
<script lang="ts">
  // Props interface
  interface Props {
    title: string;
    data: SensorData[];
    maxPoints?: number;
  }

  // Component props
  export let title: Props['title'];
  export let data: Props['data'];
  export let maxPoints: Props['maxPoints'] = 1000;

  // Reactive statements
  $: chartData = processChartData(data, maxPoints);
</script>

<div class="chart-container">
  <h2>{title}</h2>
  <!-- Chart implementation -->
</div>
```

## 🧪 **Testing Guidelines**

### **Rust Testing**

#### **Unit Tests**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_standard_payload() {
        let payload = create_test_payload();
        let decoded = decode_payload(&payload);
        
        assert!(decoded.is_some());
        if let Some(Decoded::Standard { air_temp_c, .. }) = decoded {
            assert!((air_temp_c - 25.5).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn test_decode_invalid_payload() {
        let invalid_payload = [0x00, 0x01]; // Too short
        let decoded = decode_payload(&invalid_payload);
        
        assert!(decoded.is_none());
    }
}
```

#### **Integration Tests**
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_mqtt_subscription_flow() {
        // Test complete MQTT subscription workflow
        // ... test implementation
    }
}
```

#### **Performance Tests**
```rust
#[cfg(test)]
mod bench {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_decode_payload(b: &mut Bencher) {
        let payload = create_test_payload();
        b.iter(|| decode_payload(&payload));
    }
}
```

### **Frontend Testing**

#### **Component Testing**
```typescript
// Component test example
import { render, screen } from '@testing-library/svelte';
import LiveLineChart from './LiveLineChart.svelte';

describe('LiveLineChart', () => {
  it('renders with correct title', () => {
    render(LiveLineChart, { title: 'Test Chart' });
    expect(screen.getByText('Test Chart')).toBeInTheDocument();
  });

  it('handles empty data gracefully', () => {
    render(LiveLineChart, { data: [] });
    // ... assertions
  });
});
```

#### **Integration Testing**
```typescript
// Test Tauri event handling
import { listen } from '@tauri-apps/api/event';

describe('Tauri Integration', () => {
  it('receives sensor data events', async () => {
    // ... test implementation
  });
});
```

## 🔄 **Pull Request Process**

### **Before Submitting**

1. **Ensure tests pass** locally
2. **Update documentation** if needed
3. **Check code quality** with linters
4. **Rebase** on latest main branch

### **PR Template**

```markdown
## 📝 **Description**
Brief description of changes and motivation.

## 🔧 **Changes Made**
- [ ] Feature A added
- [ ] Bug B fixed
- [ ] Performance C improved

## 🧪 **Testing**
- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Manual testing completed

## 📚 **Documentation**
- [ ] README updated
- [ ] Code comments added
- [ ] API docs updated

## 🚀 **Performance Impact**
- [ ] No performance regression
- [ ] Performance improved by X%
- [ ] Memory usage optimized

## 🔍 **Additional Notes**
Any additional context or considerations.
```

### **Review Process**

1. **Automated Checks** must pass
2. **Code Review** by maintainers
3. **Testing Verification** on target platforms
4. **Documentation Review** for completeness
5. **Final Approval** and merge

## 🐛 **Issue Reporting**

### **Bug Report Template**

```markdown
## 🐛 **Bug Description**
Clear description of the bug.

## 🔍 **Steps to Reproduce**
1. Step 1
2. Step 2
3. Step 3

## 📱 **Expected vs Actual Behavior**
- **Expected**: What should happen
- **Actual**: What actually happens

## 💻 **Environment**
- **OS**: Windows 10 / macOS / Linux
- **Rust Version**: 1.70+
- **Node Version**: 18+
- **App Version**: 0.1.0

## 📋 **Additional Context**
Screenshots, logs, or other relevant information.
```

### **Feature Request Template**

```markdown
## 🚀 **Feature Description**
Clear description of the requested feature.

## 💡 **Use Case**
Why this feature is needed and how it would be used.

## 🔧 **Proposed Implementation**
Optional: Suggestions for implementation approach.

## 📱 **Platform Considerations**
Any platform-specific requirements or limitations.
```

## 📚 **Documentation**

### **Code Documentation**

- **All public APIs** must be documented
- **Complex algorithms** require detailed explanations
- **Examples** for non-trivial functions
- **Error conditions** and handling documented

### **User Documentation**

- **Installation guides** for all platforms
- **Configuration examples** with common use cases
- **Troubleshooting guides** for common issues
- **API reference** for developers

### **Architecture Documentation**

- **System diagrams** showing component relationships
- **Data flow** documentation for complex processes
- **Performance characteristics** and optimization notes
- **Security considerations** and best practices

## 🌟 **Community**

### **Communication Channels**

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and community support
- **Discord Server**: Real-time chat and collaboration
- **Email**: Direct contact for sensitive issues

### **Contributor Recognition**

- **Contributors list** in README
- **Release notes** acknowledging contributions
- **Special thanks** for significant contributions
- **Community spotlight** for active contributors

### **Getting Help**

- **Check existing issues** before creating new ones
- **Search documentation** for answers
- **Ask in discussions** for community help
- **Tag maintainers** for urgent issues

## 📋 **Checklist for Contributors**

### **Before Contributing**
- [ ] Read and understand this guide
- [ ] Check existing issues and discussions
- [ ] Set up development environment
- [ ] Familiarize with codebase structure

### **During Development**
- [ ] Follow coding standards
- [ ] Write comprehensive tests
- [ ] Update documentation
- [ ] Test on multiple platforms

### **Before Submitting**
- [ ] All tests pass locally
- [ ] Code quality checks pass
- [ ] Documentation is complete
- [ ] PR description is clear

### **After Submission**
- [ ] Respond to review comments
- [ ] Address requested changes
- [ ] Keep PR updated with main
- [ ] Participate in discussion

---

## 🙏 **Thank You**

Thank you for contributing to **AppTest_v05**! Your contributions help make this project better for everyone in the IoT community.

**Happy coding! 🚀**

---

*This contributing guide is a living document. Please suggest improvements or clarifications through issues or discussions.*
