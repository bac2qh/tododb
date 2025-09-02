# TodoDB Demo Data Overview

This document provides an overview of the comprehensive demo data created by running `tododb --demo`.

## Demo Data Structure

### 🏗️ Major Projects (3 hierarchical project trees)

#### 1. 📱 E-commerce Website Redesign
- **Frontend Implementation**
  - Setup React + TypeScript project structure
  - Implement shopping cart component  
  - User authentication & profile management
- **Backend API Development**
  - Design database schema for products & users
  - Build REST API endpoints for product catalog

#### 2. 📱 React Native Fitness Tracker
- Setup React Native development environment
- Implement workout logging screen

#### 3. ☁️ Kubernetes Cluster Migration  
- Setup EKS cluster with Terraform
- Implement monitoring with Prometheus & Grafana

### 🧠 Personal Development Goals 2024
- 💪 Start morning exercise routine
- 📚 Read 'Atomic Habits' by James Clear  
- 💰 Research investment portfolio strategy

### 🎓 Tech Learning Roadmap
- 🦀 Learn Rust programming fundamentals
- ☁️ Study for AWS Solutions Architect certification

## Content Features Demonstrated

### Rich Markdown Content
- **Code blocks** with syntax highlighting
- **Links** to external resources (GitHub, documentation, tools)
- **Structured sections** with headings and lists  
- **Tables** and formatted data
- **Implementation details** and technical specifications

### Hierarchical Organization
- **Parent-child relationships** between todos
- **Multi-level nesting** for complex projects
- **Logical grouping** of related tasks

### Mixed Completion Status
- ~30% of tasks marked as completed
- Demonstrates both active and completed views
- Shows progress tracking capabilities

## Perfect for Demonstrations

This demo data showcases:

1. **Professional Software Development Workflows**
   - Frontend/backend development tasks
   - DevOps and infrastructure projects
   - Real-world technical requirements

2. **Personal Productivity Management**
   - Learning goals and skill development
   - Health and wellness tracking
   - Financial planning and research

3. **Rich Content Editing**
   - Markdown formatting capabilities
   - External link integration
   - Technical documentation structure

4. **App Features**
   - Hierarchical task organization
   - Search and filtering (try searching "React" or "AWS")
   - Completion status tracking
   - Glow integration for markdown editing

## Usage Tips for Screen Recording

1. **Start with tree view** to show hierarchical structure
2. **Navigate through projects** to demonstrate organization
3. **Use search functionality** to find specific topics
4. **Toggle completed view** to show progress tracking
5. **Open a todo in glow** to showcase markdown editing
6. **Demonstrate key bindings** (t for tree expansion, c for completed view, f for search)

## Resetting Demo Data

To regenerate fresh demo data:
```bash
# Remove existing database
rm ~/.local/share/tododb/todos.db

# Recreate demo data  
tododb --demo

# Launch app with fresh data
tododb
```

This ensures a clean slate for new demonstrations or testing sessions.
