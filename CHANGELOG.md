# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2024-12-19

### 🚀 **Major Release: Complete Component Refactoring & Real-time Dashboard**

#### ✨ **Added**
- **Real-time Sensor Components**: All UI components now support automatic data binding via `seriesKey` and `nodeId`
- **Comprehensive Documentation**: Added `VARIABLES.md` with complete SeriesKey reference and usage examples
- **Enhanced README**: Complete usage guide with component examples and architecture overview
- **Node-level Data Support**: Backend now exposes per-node 60-second averages to frontend
- **Responsive Dashboard**: New `+page.svelte` demonstrating all components working together

#### 🔄 **Changed**
- **SensorCard.svelte**: Complete refactor to support real-time data with automatic color coding
- **VerticalGauge.svelte**: Refactored for real-time updates with configurable thresholds
- **StatusIndicator.svelte**: Enhanced for binary sensor monitoring (rain sensors, pumps, etc.)
- **LiveLineChart.svelte**: Enhanced with comprehensive documentation and examples
- **Backend Architecture**: Added `NodeAvgUi` struct and dedicated `node_avg` event channel

#### 🏗️ **Architecture Improvements**
- **Message Passing**: Enhanced mpsc channels for node-level data distribution
- **Event Streams**: Separate `gh_avg` (greenhouse) and `node_avg` (node-specific) events
- **Component API**: Consistent `seriesKey` + `nodeId` pattern across all components
- **Type Safety**: Full TypeScript support with proper error handling

#### 📚 **Documentation**
- **VARIABLES.md**: Complete reference of all available SeriesKey values
- **Component Examples**: Usage patterns for greenhouse vs. node-specific monitoring
- **Architecture Guide**: Detailed explanation of data flow and component design
- **Best Practices**: Guidelines for optimal component configuration

#### 🎯 **Component Features**
- **Real-time Updates**: 60-second automatic data refresh from Rust backend
- **Smart Filtering**: Components automatically listen to correct event stream
- **Automatic Cleanup**: Proper event listener cleanup on component destruction
- **Responsive Design**: Mobile-first responsive layouts for all components

## [0.1.0] - 2024-12-18

### 🎉 **Initial Release: Core Infrastructure**

#### ✨ **Added**
- **Tauri Desktop App**: Cross-platform desktop application with Rust backend
- **SvelteKit Frontend**: Modern, performant web-based UI
- **MQTT Integration**: Real-time sensor data subscription and processing
- **SQLite Storage**: Persistent data storage with optimized schema
- **60-Second Aggregation**: Rolling averages for all sensor types
- **Binary Protocol**: Efficient ESP32 sensor data decoding

#### 🏗️ **Core Architecture**
- **Async Pipeline**: Tokio-based message passing architecture
- **Modular Services**: MQTT, aggregation, storage, and UI layers
- **Zero-Copy Parsing**: Efficient binary data processing
- **Bounded Channels**: Memory-safe inter-task communication

#### 📊 **Sensor Support**
- **Standard Nodes (01-04)**: Full sensor suite (temperature, humidity, PAR, pressure)
- **Outdoor Node (65001)**: Limited sensor set for external monitoring
- **Greenhouse Aggregator**: Combined averages across all active nodes
- **Real-time Charts**: LiveLineChart component with uPlot integration

#### 🔧 **Technical Features**
- **Rust Backend**: High-performance, memory-safe sensor processing
- **TypeScript Frontend**: Type-safe component development
- **Responsive Design**: Mobile and desktop optimized layouts
- **Production Ready**: Error handling, logging, and graceful degradation

---

## **Development Standards**

### **Code Quality**
- **Zero Bloat**: Clean, focused functions with single responsibilities
- **Type Safety**: Full TypeScript and Rust type coverage
- **Error Handling**: Graceful degradation and comprehensive logging
- **Performance**: Optimized for high-frequency sensor data

### **Architecture Principles**
- **Message Passing**: No locks, pure async communication
- **Modularity**: Clear separation of concerns and responsibilities
- **Scalability**: Easy to extend with new sensors and components
- **Reliability**: Bulletproof error handling and recovery

### **Performance Targets**
- **Latency**: <100ms end-to-end data processing
- **Throughput**: Support for 100+ sensors with 60-second updates
- **Memory**: Bounded memory usage with automatic cleanup
- **CPU**: Efficient async processing with minimal overhead

---

## **Migration Guide**

### **From v0.1.0 to v0.2.0**

#### **Component Usage Changes**
```svelte
<!-- OLD: Manual data management -->
<SensorCard sensorName="Temperature" sensorValue="25.5" />

<!-- NEW: Real-time automatic updates -->
<SensorCard
  title="Air Temperature"
  unit="°C"
  seriesKey="air_temp_c"
  greenhouseId={1}
  minValue={15}
  maxValue={35}
/>
```

#### **New Props Required**
- **seriesKey**: Sensor field to monitor (e.g., "vpd_kpa", "air_temp_c")
- **greenhouseId**: Target greenhouse ID (default: 1)
- **nodeId**: Specific node ID or null for greenhouse average

#### **Event Listening**
- Components automatically listen to appropriate event streams
- No manual event handling required
- Automatic cleanup on component destruction

---

## **Future Roadmap**

### **v0.3.0 (Planned)**
- **Historical Data**: Time-range queries and data export
- **Alerting System**: Configurable thresholds and notifications
- **Multi-Greenhouse**: Support for multiple greenhouse locations
- **Advanced Analytics**: Statistical analysis and trend detection

### **v0.4.0 (Planned)**
- **Mobile App**: Native mobile applications
- **Cloud Sync**: Remote monitoring and data backup
- **Machine Learning**: Predictive analytics and anomaly detection
- **API Integration**: REST API for third-party integrations

---

## **Contributing**

### **Development Workflow**
1. Fork the repository
2. Create a feature branch
3. Implement changes with comprehensive testing
4. Submit a pull request with detailed description

### **Code Standards**
- **Rust**: Follow `rustfmt` and `clippy` guidelines
- **TypeScript**: Use strict mode and proper typing
- **Documentation**: Add comprehensive doc comments
- **Testing**: Include unit and integration tests

---

## **Support & Community**

### **Getting Help**
- **GitHub Issues**: Bug reports and feature requests
- **Documentation**: Comprehensive guides and tutorials
- **Examples**: Complete usage examples in component files

### **Performance Tuning**
- **Channel Sizes**: Adjust mpsc buffer sizes for your deployment
- **Aggregation Windows**: Modify 60-second windows as needed
- **Database Optimization**: Tune SQLite settings for your hardware

---

**Built with ❤️ using Rust, Tauri, and SvelteKit for the future of IoT applications.**
