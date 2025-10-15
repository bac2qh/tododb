// use chrono::{DateTime, Duration, Utc};
use crate::database::{Database, NewTodo};
use std::collections::HashMap;

pub struct DemoDataGenerator {
    db: Database,
}

impl DemoDataGenerator {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn populate_demo_data(&self) -> anyhow::Result<()> {
        println!("🚀 Creating demo data for TodoDB...");

        // Create project hierarchies
        let project_ids = self.create_projects()?;
        
        // Add personal todos
        self.create_personal_todos()?;
        
        // Add learning todos with resources
        self.create_learning_todos()?;
        
        // Add some completed todos for demonstration
        self.mark_some_as_completed(&project_ids)?;

        println!("✅ Demo data created successfully!");
        println!("📊 You now have a variety of todos showcasing:");
        println!("   • Hierarchical project organization");
        println!("   • Markdown formatting with links");
        println!("   • Both completed and pending tasks");
        println!("   • Personal and professional todos");
        println!("   • Learning resources and goals");
        
        Ok(())
    }

    fn create_projects(&self) -> anyhow::Result<HashMap<String, i64>> {
        let mut project_ids = HashMap::new();

        // 1. Web Development Project
        let web_project_id = self.db.create_todo(NewTodo {
            title: "📱 E-commerce Website Redesign".to_string(),
            description: r#"## Project Overview
Complete redesign of the company e-commerce platform with modern UX/UI

### Goals
- Improve conversion rate by 15%
- Reduce page load time to under 2 seconds
- Implement mobile-first responsive design

### Timeline
**Deadline:** End of Q2 2024

### Resources
- [Figma Design Files](https://figma.com/project/ecommerce-redesign)
- [Project Slack Channel](https://company.slack.com/channels/web-redesign)
- [GitHub Repository](https://github.com/company/ecommerce-redesign)"#.to_string(),
            parent_id: None,
            due_by: None,
        })?;
        project_ids.insert("web_project".to_string(), web_project_id);

        // Web project subtasks
        let frontend_id = self.db.create_todo(NewTodo {
            title: "🎨 Frontend Implementation".to_string(),
            description: r#"## Frontend Development Tasks

### Technology Stack
- **Framework:** React 18 with TypeScript
- **Styling:** Tailwind CSS + Headless UI
- **State Management:** Zustand
- **Build Tool:** Vite

### Key Features to Implement
1. Product catalog with filtering
2. Shopping cart functionality  
3. User authentication flow
4. Checkout process
5. Order tracking system

### Performance Requirements
- First Contentful Paint < 1.5s
- Largest Contentful Paint < 2.5s
- Cumulative Layout Shift < 0.1"#.to_string(),
            parent_id: Some(web_project_id),
            due_by: None,
        })?;

        self.db.create_todo(NewTodo {
            title: "🔧 Setup React + TypeScript project structure".to_string(),
            description: r#"## Project Setup Checklist

### Initial Setup
- [x] Create Vite project with React + TypeScript template
- [ ] Configure ESLint and Prettier
- [ ] Setup Husky pre-commit hooks
- [ ] Configure Tailwind CSS
- [ ] Setup testing environment (Vitest + Testing Library)

### Project Structure
```
src/
├── components/
│   ├── ui/          # Reusable UI components
│   ├── forms/       # Form components
│   └── layout/      # Layout components
├── pages/           # Route components
├── hooks/           # Custom React hooks
├── stores/          # Zustand stores
├── types/           # TypeScript type definitions
└── utils/           # Utility functions
```

### References
- [Vite React TypeScript Template](https://vitejs.dev/guide/)
- [Tailwind CSS Installation](https://tailwindcss.com/docs/installation)
- [Zustand Documentation](https://docs.pmnd.rs/zustand/getting-started/introduction)"#.to_string(),
            parent_id: Some(frontend_id),
            due_by: None,
        })?;

        self.db.create_todo(NewTodo {
            title: "🛒 Implement shopping cart component".to_string(),
            description: r#"## Shopping Cart Implementation

### Requirements
- Add/remove items from cart
- Update item quantities
- Calculate totals with tax
- Persist cart state across sessions
- Smooth animations for cart updates

### Components to Build
1. `CartProvider` - Context for cart state
2. `CartDrawer` - Slide-out cart interface  
3. `CartItem` - Individual cart item component
4. `CartSummary` - Order totals and checkout button

### State Management
Use Zustand store to manage:
- Cart items array
- Total quantities and prices
- Loading states for async operations

### Animations
Use Framer Motion for:
- Cart drawer slide animations
- Item add/remove animations
- Quantity change transitions

### Testing
- Unit tests for cart logic
- Integration tests for cart flow
- Accessibility tests (keyboard navigation)

### Design Reference
[Cart Component Figma](https://figma.com/file/cart-component)"#.to_string(),
            parent_id: Some(frontend_id),
            due_by: None,
        })?;

        self.db.create_todo(NewTodo {
            title: "🔐 User authentication & profile management".to_string(),
            description: r#"## Authentication System

### Features Required
- Email/password registration and login
- OAuth integration (Google, GitHub)
- Password reset functionality
- Email verification
- Profile editing
- Account deletion

### Security Requirements
- JWT token-based authentication
- Secure password hashing (bcrypt)
- Rate limiting on auth endpoints
- CSRF protection
- Input validation and sanitization

### User Profile Features
- Avatar upload and cropping
- Personal information management
- Order history view
- Address book management
- Notification preferences

### Implementation Notes
- Use NextAuth.js for OAuth integration
- Implement proper error handling
- Add loading states for all auth operations
- Ensure mobile responsiveness

### API Endpoints
```
POST /api/auth/register
POST /api/auth/login  
POST /api/auth/logout
POST /api/auth/reset-password
GET  /api/user/profile
PUT  /api/user/profile
DELETE /api/user/account
```

### Resources
- [NextAuth.js Documentation](https://next-auth.js.org/)
- [JWT Best Practices](https://tools.ietf.org/html/rfc8725)"#.to_string(),
            parent_id: Some(frontend_id),
            due_by: None,
        })?;

        // Backend tasks
        let backend_id = self.db.create_todo(NewTodo {
            title: "⚙️ Backend API Development".to_string(),
            description: r#"## Backend Development

### Technology Stack
- **Runtime:** Node.js 18+
- **Framework:** Express.js with TypeScript
- **Database:** PostgreSQL with Prisma ORM
- **Authentication:** JWT + bcrypt
- **File Storage:** AWS S3
- **Caching:** Redis

### API Design Principles
- RESTful endpoints with proper HTTP methods
- Consistent error response format
- Request/response validation with Joi
- API rate limiting
- Comprehensive logging with Winston

### Database Design
- Normalized schema design
- Proper indexing for performance
- Migration strategy for schema changes
- Backup and recovery procedures"#.to_string(),
            parent_id: Some(web_project_id),
            due_by: None,
        })?;

        self.db.create_todo(NewTodo {
            title: "🗄️ Design database schema for products & users".to_string(),
            description: r#"## Database Schema Design

### Core Entities
1. **Users** - Customer accounts and authentication
2. **Products** - Product catalog and inventory
3. **Orders** - Order management and tracking  
4. **Categories** - Product categorization
5. **Reviews** - Customer product reviews

### Schema Highlights
```sql
-- Users table
CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email VARCHAR(255) UNIQUE NOT NULL,
  password_hash VARCHAR(255) NOT NULL,
  first_name VARCHAR(100),
  last_name VARCHAR(100),
  avatar_url TEXT,
  email_verified BOOLEAN DEFAULT false,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

-- Products table  
CREATE TABLE products (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name VARCHAR(255) NOT NULL,
  description TEXT,
  price DECIMAL(10,2) NOT NULL,
  inventory_count INTEGER DEFAULT 0,
  category_id UUID REFERENCES categories(id),
  images JSONB DEFAULT '[]',
  is_active BOOLEAN DEFAULT true,
  created_at TIMESTAMP DEFAULT NOW()
);
```

### Indexing Strategy
- Email index for fast user lookups
- Category + price composite index for product filtering
- Full-text search index on product names and descriptions

### Performance Considerations
- Connection pooling configuration
- Query optimization for product searches
- Pagination for large result sets

### Migration Plan
Use Prisma migrations for schema versioning and deployment

### Tools
- [Prisma Schema Reference](https://www.prisma.io/docs/reference/api-reference/prisma-schema-reference)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [Database Design Tool](https://dbdiagram.io/)"#.to_string(),
            parent_id: Some(backend_id),
            due_by: None,
        })?;

        self.db.create_todo(NewTodo {
            title: "🔌 Build REST API endpoints for product catalog".to_string(),
            description: r#"## Product Catalog API

### Required Endpoints

#### Product Management
```
GET    /api/products              # List products with filtering
GET    /api/products/:id          # Get single product details
POST   /api/products              # Create new product (admin)
PUT    /api/products/:id          # Update product (admin)  
DELETE /api/products/:id          # Delete product (admin)
```

#### Category Management
```
GET    /api/categories            # List all categories
GET    /api/categories/:id/products # Products by category
```

#### Search & Filtering
```
GET    /api/products/search?q={query}
GET    /api/products?category={id}&min_price={amount}&max_price={amount}
```

### Query Parameters
- `page` - Pagination page number
- `limit` - Items per page (default: 20, max: 100)
- `sort` - Sort by: price_asc, price_desc, name, newest
- `category` - Filter by category ID
- `min_price` / `max_price` - Price range filtering

### Response Format
```json
{
  "data": [...],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 150,
    "totalPages": 8
  }
}
```

### Error Handling
- Proper HTTP status codes
- Consistent error message format
- Input validation with helpful messages
- Rate limiting responses

### Performance Features
- Response caching with Redis
- Database query optimization
- Image URL generation for CDN
- Bulk operations support

### Testing Strategy
- Unit tests for business logic
- Integration tests for API endpoints
- Load testing for performance validation

### Documentation
Generate API docs with Swagger/OpenAPI specification

### References
- [Express.js Best Practices](https://expressjs.com/en/advanced/best-practice-performance.html)
- [REST API Design Guidelines](https://restfulapi.net/)
- [HTTP Status Codes](https://httpstatuses.com/)"#.to_string(),
            parent_id: Some(backend_id),
            due_by: None,
        })?;

        // 2. Mobile App Project
        let mobile_project_id = self.db.create_todo(NewTodo {
            title: "📱 React Native Fitness Tracker".to_string(),
            description: r#"## Fitness Tracker Mobile App

### Project Vision
Create a comprehensive fitness tracking app that helps users monitor workouts, nutrition, and progress toward their health goals.

### Core Features
- **Workout Logging** - Track exercises, sets, reps, and weights
- **Nutrition Tracking** - Log meals and monitor macronutrients  
- **Progress Analytics** - Charts and insights on fitness journey
- **Social Features** - Share achievements and connect with friends
- **Wearable Integration** - Sync with Apple Health and Google Fit

### Technology Stack
- **Framework:** React Native with TypeScript
- **State Management:** Redux Toolkit
- **Navigation:** React Navigation 6
- **Styling:** NativeWind (Tailwind for React Native)
- **Backend:** Supabase for auth and data
- **Charts:** Victory Native
- **Testing:** Jest + React Native Testing Library

### Target Platforms
- iOS 14+ and Android API 24+ (Android 7.0+)
- Tablet support for larger screens
- Apple Watch and Wear OS companion apps (future phase)

### Timeline
**MVP Target:** 3 months
**Full Release:** 6 months

### Resources
- [Design Mockups](https://figma.com/fitness-tracker-app)
- [Product Requirements Doc](https://notion.so/fitness-tracker-prd)
- [Technical Architecture](https://miro.com/fitness-app-architecture)"#.to_string(),
            parent_id: None,
            due_by: None,
        })?;
        project_ids.insert("mobile_project".to_string(), mobile_project_id);

        // Mobile subtasks
        self.db.create_todo(NewTodo {
            title: "⚡ Setup React Native development environment".to_string(),
            description: r#"## Development Environment Setup

### Prerequisites Installation
- **Node.js 18+** - JavaScript runtime
- **React Native CLI** - Development toolchain
- **Android Studio** - Android development (includes SDK and emulator)
- **Xcode** - iOS development (macOS only)
- **CocoaPods** - iOS dependency manager

### Project Initialization
```bash
# Create new React Native project with TypeScript
npx react-native@latest init FitnessTracker --template react-native-template-typescript

# Install additional dependencies
npm install @reduxjs/toolkit react-redux
npm install @react-navigation/native @react-navigation/bottom-tabs
npm install nativewind tailwindcss
npm install react-native-vector-icons
```

### Development Setup
1. Configure Metro bundler for custom fonts and assets
2. Setup ESLint and Prettier for code formatting
3. Configure Flipper for debugging
4. Setup Reactotron for Redux debugging
5. Configure testing environment

### Platform-Specific Configuration

#### iOS Setup
- Configure Info.plist for permissions
- Setup app icons and launch screens  
- Configure signing certificates
- Test on iOS Simulator

#### Android Setup
- Configure AndroidManifest.xml permissions
- Setup app icons and splash screens
- Configure Proguard for release builds
- Test on Android emulator

### Helpful Commands
```bash
# Start Metro bundler
npx react-native start

# Run on iOS simulator  
npx react-native run-ios

# Run on Android emulator
npx react-native run-android

# Clear cache if needed
npx react-native start --reset-cache
```

### Resources
- [React Native Environment Setup](https://reactnative.dev/docs/environment-setup)
- [TypeScript with React Native](https://reactnative.dev/docs/typescript)
- [NativeWind Setup Guide](https://www.nativewind.dev/quick-starts/react-native-cli)"#.to_string(),
            parent_id: Some(mobile_project_id),
            due_by: None,
        })?;

        self.db.create_todo(NewTodo {
            title: "💪 Implement workout logging screen".to_string(),
            description: r#"## Workout Logging Interface

### Screen Components
1. **Exercise Selection** - Search and browse exercise database
2. **Set Tracking** - Add sets with reps, weight, and rest time
3. **Timer Integration** - Rest timer between sets
4. **Workout Summary** - Review and save completed workout
5. **History View** - Previous workout data for reference

### Features to Implement
- Exercise database with 200+ exercises
- Custom exercise creation
- Workout templates for quick start
- Progress tracking (weight/reps progression)
- Auto-suggestions based on workout history
- Photo/video exercise demonstrations

### State Management
Redux store structure:
```typescript
interface WorkoutState {
  currentWorkout: {
    exercises: Exercise[];
    startTime: Date;
    isActive: boolean;
  };
  exerciseDatabase: Exercise[];
  workoutHistory: CompletedWorkout[];
  templates: WorkoutTemplate[];
}
```

### UI/UX Considerations
- Quick input methods (number pad, gesture controls)
- Visual progress indicators
- Haptic feedback for completed sets
- Dark mode support for gym environments
- Large touch targets for easy use during workouts

### Data Model
```typescript
interface Exercise {
  id: string;
  name: string;
  category: 'strength' | 'cardio' | 'flexibility';
  muscleGroups: string[];
  instructions: string;
  videoUrl?: string;
}

interface WorkoutSet {
  reps: number;
  weight?: number;  // for strength exercises
  duration?: number; // for cardio/timed exercises
  restTime?: number;
  completed: boolean;
}
```

### Offline Support
- Cache exercise database locally
- Queue workout data for sync when online
- Conflict resolution for data synchronization

### Testing Strategy
- Component tests for UI interactions
- Redux store testing for state management  
- Integration tests for workout flow
- Accessibility testing for screen readers

### Performance Optimization
- Lazy load exercise database
- Optimize list rendering with FlatList
- Image caching for exercise demonstrations
- Debounce search input

### References
- [Exercise Database API](https://rapidapi.com/exercisedb/api/exercisedb)
- [React Native AsyncStorage](https://react-native-async-storage.github.io/async-storage/)
- [Victory Native Charts](https://formidable.com/open-source/victory/docs/native/)"#.to_string(),
            parent_id: Some(mobile_project_id),
            due_by: None,
        })?;

        // 3. DevOps Project  
        let devops_project_id = self.db.create_todo(NewTodo {
            title: "☁️ Kubernetes Cluster Migration".to_string(),
            description: r#"## Infrastructure Modernization Project

### Project Goals
Migrate existing Docker Swarm infrastructure to Kubernetes for better scalability, observability, and automated operations.

### Current State
- 15 microservices running on Docker Swarm
- Manual deployment processes
- Limited monitoring and logging
- Single point of failure concerns

### Target Architecture
- **Platform:** Amazon EKS (Elastic Kubernetes Service)
- **Service Mesh:** Istio for traffic management
- **Monitoring:** Prometheus + Grafana
- **Logging:** ELK Stack (Elasticsearch, Logstash, Kibana)
- **CI/CD:** GitLab CI with ArgoCD for GitOps
- **Secret Management:** AWS Secrets Manager + External Secrets Operator

### Success Metrics
- 99.9% uptime SLA achievement
- 50% reduction in deployment time
- Automated scaling and recovery
- Complete observability across all services
- Zero-downtime deployments

### Timeline
**Phase 1:** Infrastructure Setup (4 weeks)
**Phase 2:** Service Migration (6 weeks)  
**Phase 3:** Monitoring & Optimization (3 weeks)

### Budget Considerations
- EKS cluster costs: ~$200/month
- Monitoring stack: ~$150/month
- Storage and networking: ~$100/month
- Training and certification: $5,000

### Risk Mitigation
- Maintain parallel environments during migration
- Comprehensive testing in staging
- Rollback procedures for each service
- Team training on Kubernetes operations"#.to_string(),
            parent_id: None,
            due_by: None,
        })?;
        project_ids.insert("devops_project".to_string(), devops_project_id);

        self.db.create_todo(NewTodo {
            title: "🏗️ Setup EKS cluster with Terraform".to_string(),
            description: r#"## EKS Cluster Infrastructure

### Terraform Configuration
Create infrastructure as code for reproducible environments:

```hcl
# main.tf structure
- VPC and networking (subnets, IGW, NAT gateways)
- EKS cluster with managed node groups  
- IAM roles and policies
- Security groups and NACLs
- RDS database for stateful services
- ElastiCache for Redis caching
- S3 buckets for backups and artifacts
```

### Cluster Specifications
- **Kubernetes Version:** 1.28+
- **Node Groups:** 
  - General workloads: t3.medium (2-10 nodes)
  - CPU-intensive: c5.large (1-5 nodes)
  - Memory-intensive: r5.large (1-3 nodes)
- **Networking:** CNI with AWS VPC CNI plugin
- **Storage Classes:** EBS CSI driver with gp3 volumes

### Security Configuration
- Enable EKS cluster endpoint private access
- Configure Pod Security Standards (restricted)
- Network policies for micro-segmentation
- RBAC with least-privilege principles
- Enable audit logging to CloudWatch

### Add-on Installation
```bash
# Essential cluster add-ons
kubectl apply -f https://raw.githubusercontent.com/aws/aws-load-balancer-controller/v2.4.1/docs/install/iam_policy.json

# Install AWS Load Balancer Controller
helm repo add eks https://aws.github.io/eks-charts
helm install aws-load-balancer-controller eks/aws-load-balancer-controller

# Install EBS CSI driver
kubectl apply -k "github.com/kubernetes-sigs/aws-ebs-csi-driver/deploy/kubernetes/overlays/stable/?ref=master"

# Install Cluster Autoscaler
kubectl apply -f https://raw.githubusercontent.com/kubernetes/autoscaler/master/cluster-autoscaler/cloudprovider/aws/examples/cluster-autoscaler-autodiscover.yaml
```

### Cost Optimization
- Use Spot instances for non-critical workloads
- Configure Horizontal Pod Autoscaler (HPA)
- Implement Vertical Pod Autoscaler (VPA)
- Set resource requests/limits appropriately
- Enable cluster autoscaler for node optimization

### Backup Strategy
- etcd backup to S3 with Velero
- Persistent volume snapshots
- Configuration backup to Git repository
- Disaster recovery procedures documentation

### Validation Checklist
- [ ] Cluster API accessible via kubectl
- [ ] Node groups healthy and auto-scaling
- [ ] Load balancer controller functional
- [ ] Storage classes available
- [ ] Network policies enforced
- [ ] Monitoring add-ons deployed
- [ ] Backup procedures tested

### Documentation Requirements
- Network architecture diagrams
- Security configuration details  
- Operational runbooks
- Troubleshooting guides
- Cost monitoring dashboards

### References
- [EKS Best Practices Guide](https://aws.github.io/aws-eks-best-practices/)
- [Terraform EKS Module](https://registry.terraform.io/modules/terraform-aws-modules/eks/aws/latest)
- [Kubernetes Security Checklist](https://kubernetes.io/docs/concepts/security/security-checklist/)"#.to_string(),
            parent_id: Some(devops_project_id),
            due_by: None,
        })?;

        self.db.create_todo(NewTodo {
            title: "📊 Implement monitoring with Prometheus & Grafana".to_string(),
            description: r#"## Observability Stack Implementation

### Monitoring Architecture
Deploy comprehensive monitoring using the kube-prometheus-stack:

```yaml
# Key components to deploy
- Prometheus Operator
- Grafana with pre-configured dashboards
- Alertmanager for notifications
- Node Exporter for host metrics
- kube-state-metrics for K8s object metrics
- Blackbox exporter for endpoint monitoring
```

### Prometheus Configuration
```yaml
# Custom ServiceMonitors for applications
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: app-metrics
spec:
  selector:
    matchLabels:
      app: my-application
  endpoints:
  - port: metrics
    interval: 30s
    path: /metrics
```

### Essential Dashboards
1. **Cluster Overview** - Node health, resource usage, pod status
2. **Application Performance** - Response times, throughput, error rates
3. **Infrastructure Metrics** - CPU, memory, disk, network utilization  
4. **Business Metrics** - Custom KPIs and SLIs
5. **Security Monitoring** - Failed authentication, suspicious activity

### Alerting Rules
```yaml
# Critical alerts to configure
- HighCPUUsage (>80% for 5 minutes)
- HighMemoryUsage (>85% for 5 minutes)
- PodCrashLooping (restarts >5 in 10 minutes)
- DeploymentReplicasMismatch (desired != available)
- PersistentVolumeUsage (>90% full)
- CertificateExpiry (expires in <30 days)
- DatabaseConnectionFailure
- HighErrorRate (>5% for 2 minutes)
```

### Notification Channels
- **Slack** - Real-time alerts for on-call team
- **PagerDuty** - Critical alerts for immediate response
- **Email** - Weekly/monthly reports and summaries
- **Webhook** - Integration with ticketing systems

### Data Retention Strategy
- **High-resolution metrics:** 15 days (15s intervals)
- **Medium-resolution:** 90 days (5m intervals)  
- **Long-term storage:** 2 years (1h intervals)
- **Cold storage:** S3 with Thanos for historical data

### Custom Metrics Implementation
```go
// Example application metrics in Go
import "github.com/prometheus/client_golang/prometheus"

var (
    httpRequestsTotal = prometheus.NewCounterVec(
        prometheus.CounterOpts{
            Name: "http_requests_total",
            Help: "Total number of HTTP requests.",
        },
        []string{"method", "endpoint", "status_code"},
    )
    
    httpRequestDuration = prometheus.NewHistogramVec(
        prometheus.HistogramOpts{
            Name: "http_request_duration_seconds",
            Help: "Duration of HTTP requests.",
        },
        []string{"method", "endpoint"},
    )
)
```

### SLI/SLO Configuration
Define Service Level Indicators and Objectives:
- **Availability SLO:** 99.9% uptime (43.2 minutes downtime/month)
- **Latency SLO:** 95th percentile response time <500ms
- **Error Rate SLO:** <0.1% error rate for user-facing requests
- **Throughput SLO:** Handle 1000 requests/second peak load

### Grafana Setup
- Configure LDAP/OAuth authentication
- Create team-based folder permissions
- Import community dashboards for common services
- Setup automated PDF reports for stakeholders
- Configure data source high availability

### Performance Optimization
- Use recording rules for expensive queries
- Configure appropriate scrape intervals
- Implement metric relabeling for cost control
- Use Grafana query caching
- Optimize dashboard queries with variables

### Backup and Disaster Recovery
- Grafana dashboard backup to Git repository
- Prometheus configuration backup
- Historical data backup with Thanos
- Alert rule version control
- Recovery procedures documentation

### Security Considerations
- Enable TLS for all metric endpoints
- Configure network policies for monitoring namespace
- Use service accounts with minimal permissions
- Secure Grafana admin credentials
- Audit logging for configuration changes

### References
- [Prometheus Best Practices](https://prometheus.io/docs/practices/naming/)
- [Grafana Dashboard Best Practices](https://grafana.com/docs/grafana/latest/best-practices/)
- [SRE Book - Monitoring Distributed Systems](https://sre.google/sre-book/monitoring-distributed-systems/)"#.to_string(),
            parent_id: Some(devops_project_id),
            due_by: None,
        })?;

        Ok(project_ids)
    }

    fn create_personal_todos(&self) -> anyhow::Result<()> {
        // Personal development
        let personal_dev_id = self.db.create_todo(NewTodo {
            title: "🧠 Personal Development Goals 2024".to_string(),
            description: r#"## Annual Personal Growth Plan

### Professional Skills
- Complete AWS Solutions Architect certification
- Learn Rust programming language fundamentals
- Improve public speaking and presentation skills
- Build personal project portfolio on GitHub

### Health & Wellness
- Exercise consistently (4x per week minimum)
- Practice meditation daily (15 minutes)  
- Read 24 books this year (2 per month)
- Maintain work-life balance

### Financial Goals
- Max out 401(k) contributions
- Build 6-month emergency fund
- Research and invest in index funds
- Track expenses with budgeting app

### Personal Projects
- Contribute to open-source projects
- Write technical blog posts monthly
- Learn photography and photo editing
- Plan and take dream vacation to Japan

### Relationships
- Schedule regular family video calls
- Maintain friendships with monthly activities
- Network with other developers in local community
- Mentor junior developers at work"#.to_string(),
            parent_id: None,
            due_by: None,
        })?;

        // Health subtasks
        self.db.create_todo(NewTodo {
            title: "💪 Start morning exercise routine".to_string(),
            description: r#"## Morning Workout Plan

### Goals
- Build consistent exercise habit
- Increase energy levels throughout the day
- Improve strength and cardiovascular health
- Better sleep quality

### Routine Structure (30 minutes)
**Monday/Wednesday/Friday - Strength Training**
- 5 min warm-up (jumping jacks, arm circles)
- 20 min bodyweight exercises:
  - Push-ups: 3 sets of 8-12 reps
  - Squats: 3 sets of 15-20 reps  
  - Plank: 3 sets of 30-60 seconds
  - Lunges: 3 sets of 10 per leg
- 5 min cool-down and stretching

**Tuesday/Thursday - Cardio**
- 25 min moderate intensity workout:
  - Option 1: Brisk walk/jog outdoors
  - Option 2: YouTube fitness video
  - Option 3: Cycling or elliptical
- 5 min stretching

**Weekend - Active Recovery**
- Yoga or gentle stretching
- Hiking or recreational activities
- Sports with friends/family

### Equipment Needed
- Exercise mat
- Resistance bands (optional)
- Dumbbells or water bottles for weights
- Comfortable workout clothes
- Athletic shoes

### Progress Tracking
- Weekly photos for visual progress
- Fitness app to log workouts
- Note energy levels and mood
- Track improvements in reps/duration

### Motivation Strategies  
- Lay out workout clothes the night before
- Find workout buddy or accountability partner
- Reward weekly consistency milestones
- Listen to energizing music or podcasts
- Focus on how good exercise makes you feel

### Helpful Resources
- [Fitness Blender YouTube Channel](https://www.youtube.com/user/FitnessBlender)
- [7 Minute Workout App](https://apps.apple.com/us/app/seven-7-minute-workout/id650627525)
- [MyFitnessPal](https://www.myfitnesspal.com/) for nutrition tracking"#.to_string(),
            parent_id: Some(personal_dev_id),
            due_by: None,
        })?;

        self.db.create_todo(NewTodo {
            title: "📚 Read 'Atomic Habits' by James Clear".to_string(),
            description: r#"## Book Study Plan

### About the Book
**"Atomic Habits" by James Clear**
- Focus: Building good habits and breaking bad ones
- Key concept: 1% improvements compound over time
- Practical strategies for habit formation
- Based on scientific research and real-world examples

### Reading Schedule
**Target:** Complete in 3 weeks (1 chapter per day)

**Week 1: The Fundamentals**
- Chapter 1: The Surprising Power of Atomic Habits
- Chapter 2: How Your Habits Shape Your Identity  
- Chapter 3: How to Build Better Habits in 4 Simple Steps
- Chapter 4: The Man Who Didn't Look Right

**Week 2: Make It Obvious & Attractive** 
- Chapters 5-11: The 1st and 2nd Laws of Behavior Change
- Focus on habit stacking and environment design
- Learn about temptation bundling

**Week 3: Make It Easy & Satisfying**
- Chapters 12-20: The 3rd and 4th Laws of Behavior Change  
- Understand the importance of starting small
- Master habit tracking and accountability

### Key Takeaways to Apply
1. **Identity-based habits** - Focus on who you want to become
2. **Environment design** - Make good habits obvious, bad habits invisible
3. **The 2-minute rule** - Start new habits with just 2 minutes
4. **Habit stacking** - Link new habits to existing ones
5. **Systems vs. goals** - Focus on the process, not just outcomes

### Implementation Plan
- Choose 2-3 habits to implement using book principles
- Design environment to support new habits
- Track progress using habit tracker app
- Review and adjust approach weekly

### Discussion Questions
- How do current habits align with desired identity?
- What environmental changes could support better habits?
- Which habits deserve the most focus for maximum impact?

### Follow-up Actions
- Write summary blog post about key learnings
- Share insights with accountability partner
- Apply habit stacking to morning routine
- Redesign workspace for better productivity habits

### Related Resources
- [James Clear's Blog](https://jamesclear.com/articles)
- [Habit Tracker Apps](https://www.habitica.com/)
- [TED Talk: The Power of Small Wins](https://www.ted.com/talks/bj_fogg_tiny_habits_the_small_changes_that_change_everything)"#.to_string(),
            parent_id: Some(personal_dev_id),
            due_by: None,
        })?;

        // Finance task
        self.db.create_todo(NewTodo {
            title: "💰 Research investment portfolio strategy".to_string(),
            description: r#"## Investment Research & Planning

### Current Financial Situation
- Emergency fund: 3 months expenses (goal: 6 months)
- 401(k): Contributing 6% with company match
- Student loans: $15,000 remaining at 4.2% interest
- Monthly surplus for investing: $800-1,200

### Investment Goals
**Short-term (1-2 years)**
- Build emergency fund to 6 months
- Max out Roth IRA contributions ($6,000/year)

**Medium-term (3-10 years)**  
- Save for house down payment ($50,000)
- Increase 401(k) contributions to 15%

**Long-term (10+ years)**
- Build wealth for early retirement (FIRE strategy)
- Achieve financial independence by age 50

### Portfolio Allocation Research
**Aggressive Growth Portfolio (Age 28)**
- 80% Stocks / 20% Bonds
- Focus on low-cost index funds
- International diversification (30% international stocks)
- Consider target-date funds for simplicity

**Fund Research List**
- VTSAX (Vanguard Total Stock Market)
- VTIAX (Vanguard Total International Stock)  
- VBTLX (Vanguard Total Bond Market)
- Target Date 2060 funds comparison

### Investment Platforms to Compare
1. **Vanguard** - Low fees, excellent index funds
2. **Fidelity** - Zero-fee index funds, good mobile app
3. **Schwab** - Comprehensive platform, good research tools
4. **M1 Finance** - Automated rebalancing, fractional shares

### Key Metrics to Research
- Expense ratios (target: <0.20%)
- Historical performance vs. benchmarks
- Fund size and liquidity
- Tax efficiency of fund structure
- Minimum investment requirements

### Risk Management
- Dollar-cost averaging strategy
- Rebalancing frequency (quarterly vs. annual)
- Tax-loss harvesting opportunities
- Asset location optimization (tax-advantaged vs. taxable)

### Education Resources
- **Books to Read:**
  - "A Random Walk Down Wall Street" - Burton Malkiel
  - "The Bogleheads' Guide to Investing" - Taylor Larimore
  - "Your Money or Your Life" - Vicki Robin

- **Websites & Tools:**
  - [Bogleheads.org](https://www.bogleheads.org/) community
  - [Portfolio Visualizer](https://www.portfoliovisualizer.com/) for backtesting
  - [Morningstar.com](https://www.morningstar.com/) for fund research

### Action Items
1. Compare expense ratios of target funds
2. Calculate tax implications of different accounts
3. Model portfolio performance scenarios
4. Set up automatic investment transfers
5. Create investment policy statement
6. Schedule quarterly portfolio reviews

### Tax Optimization Strategy
- Prioritize tax-advantaged accounts (401k, IRA)
- Hold tax-inefficient funds in tax-deferred accounts
- Consider Roth conversions in low-income years
- Harvest tax losses annually in taxable accounts

### Timeline
**Week 1:** Research funds and platforms
**Week 2:** Open investment accounts  
**Week 3:** Set up automatic investing
**Week 4:** Create tracking spreadsheet and review schedule"#.to_string(),
            parent_id: Some(personal_dev_id),
            due_by: None,
        })?;

        Ok(())
    }

    fn create_learning_todos(&self) -> anyhow::Result<()> {
        let learning_id = self.db.create_todo(NewTodo {
            title: "🎓 Tech Learning Roadmap".to_string(),
            description: r#"## 2024 Technical Skill Development

### Core Programming Languages
- **Rust** - Systems programming and performance
- **TypeScript** - Frontend and backend development  
- **Python** - Data science and automation
- **Go** - Microservices and cloud-native development

### Cloud & DevOps Skills
- AWS Solutions Architect certification
- Kubernetes administration (CKA)
- Infrastructure as Code (Terraform)
- CI/CD pipeline optimization
- Observability and monitoring

### Frontend Development
- React 18 with concurrent features
- Next.js 13+ with app directory
- CSS-in-JS and Tailwind CSS mastery
- Web performance optimization
- Progressive Web Apps (PWA)

### Backend & Databases
- Microservices architecture patterns
- Event-driven architecture
- PostgreSQL advanced features
- Redis caching strategies
- GraphQL API design

### Learning Resources
- [Frontend Masters](https://frontendmasters.com/) courses
- [A Cloud Guru](https://acloudguru.com/) for AWS
- [The Rust Programming Language Book](https://doc.rust-lang.org/book/)
- [Kubernetes Official Documentation](https://kubernetes.io/docs/)
- [System Design Primer](https://github.com/donnemartin/system-design-primer)

### Time Investment
- 1 hour daily for focused learning
- Weekend projects for hands-on practice
- Monthly tech meetups and conferences
- Quarterly skill assessments and goal adjustments"#.to_string(),
            parent_id: None,
            due_by: None,
        })?;

        self.db.create_todo(NewTodo {
            title: "🦀 Learn Rust programming fundamentals".to_string(),
            description: r#"## Rust Learning Path

### Why Rust?
- Memory safety without garbage collection
- High performance systems programming
- Growing ecosystem and job market
- Excellent for CLI tools, web servers, and blockchain

### Learning Resources
**Primary Resource:** [The Rust Programming Language Book](https://doc.rust-lang.org/book/)

**Supplementary Resources:**
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings Exercises](https://github.com/rust-lang/rustlings)
- [The Rust Reference](https://doc.rust-lang.org/reference/)
- [Awesome Rust](https://github.com/rust-unofficial/awesome-rust) - curated resources

### Week-by-Week Study Plan

**Week 1-2: Basics**
- Installation and Hello World
- Variables, data types, and functions
- Control flow (if/else, loops)
- Ownership and borrowing concepts

**Week 3-4: Core Concepts**  
- Structs and enums
- Pattern matching with match
- Error handling (Result and Option)
- Collections (Vec, HashMap)

**Week 5-6: Advanced Features**
- Traits and generics
- Lifetimes and references
- Modules and packages (Cargo)
- Testing with `#[test]`

**Week 7-8: Real Projects**
- CLI application with clap
- Web server with Axum or Warp
- File processing utility
- Simple HTTP client

### Practice Projects
1. **Command Line Calculator**
   - Parse mathematical expressions
   - Handle errors gracefully
   - Support basic operations (+, -, *, /, ^)

2. **File Organizer Tool**
   - Read directory contents
   - Sort files by type/date/size
   - Move files to organized folders

3. **Web Scraper**
   - HTTP requests with reqwest
   - HTML parsing with scraper
   - Concurrent processing with tokio

4. **Todo CLI Application**
   - JSON file storage
   - CRUD operations
   - Search and filtering

### Key Concepts to Master
- **Ownership System** - Understanding borrowing, lifetimes, and moves
- **Error Handling** - Using Result<T, E> and Option<T> effectively  
- **Pattern Matching** - Leveraging match expressions and destructuring
- **Traits** - Defining shared behavior and using standard library traits
- **Async Programming** - Using async/await with tokio runtime

### Community and Practice
- Join [Rust Users Forum](https://users.rust-lang.org/)
- Participate in [r/rust](https://www.reddit.com/r/rust/) discussions
- Contribute to open source Rust projects
- Attend local Rust meetups or online events

### Assessment Milestones
- [ ] Complete first 10 chapters of Rust Book
- [ ] Finish all Rustlings exercises  
- [ ] Build and deploy one CLI tool
- [ ] Contribute to an open source Rust project
- [ ] Write blog post about Rust learning experience

### Time Commitment
- **Daily:** 45-60 minutes reading and coding
- **Weekly:** 2-3 hours on practice projects
- **Monthly:** Review progress and adjust learning plan

### Next Steps After Fundamentals
- Web development with Axum or Rocket
- Systems programming projects
- WebAssembly (WASM) development
- Blockchain development with Substrate

### Useful VSCode Extensions
- rust-analyzer (official Rust support)
- CodeLLDB (debugging support)
- Better TOML (Cargo.toml syntax highlighting)
- Error Lens (inline error display)"#.to_string(),
            parent_id: Some(learning_id),
            due_by: None,
        })?;

        self.db.create_todo(NewTodo {
            title: "☁️ Study for AWS Solutions Architect certification".to_string(),
            description: r#"## AWS Solutions Architect Associate (SAA-C03)

### Certification Overview
- **Exam Code:** SAA-C03
- **Duration:** 130 minutes
- **Questions:** 65 multiple choice/multiple response
- **Passing Score:** ~720/1000 (72%)
- **Cost:** $150 USD
- **Validity:** 3 years

### Exam Domains & Weights
1. **Design Resilient Architectures (26%)**
   - Multi-tier architecture design
   - High availability and disaster recovery
   - Decoupling mechanisms

2. **Design High-Performing Architectures (24%)**
   - Storage solutions and data access patterns
   - Caching strategies
   - Compute solutions

3. **Design Secure Architectures (30%)**  
   - Identity and access management
   - Network security
   - Data protection

4. **Design Cost-Optimized Architectures (20%)**
   - Cost-effective storage solutions
   - Compute optimization
   - Database cost optimization

### Study Resources

**Primary Resources:**
- [AWS Well-Architected Framework](https://aws.amazon.com/architecture/well-architected/)
- [AWS Solutions Architect Associate Exam Guide](https://d1.awsstatic.com/training-and-certification/docs-sa-assoc/AWS-Certified-Solutions-Architect-Associate_Exam-Guide.pdf)
- [AWS Whitepapers](https://aws.amazon.com/whitepapers/) - focus on architectural best practices

**Video Courses:**
- A Cloud Guru SAA-C03 course
- Stephane Maarek's Ultimate AWS Certified Solutions Architect Associate course (Udemy)
- Adrian Cantrill's AWS Solutions Architect Associate course

**Practice Exams:**
- Jon Bonso Practice Tests (Tutorials Dojo)
- Whizlabs AWS practice exams
- AWS official practice exam

### Core AWS Services to Master

**Compute:**
- EC2 (instance types, placement groups, user data)
- Auto Scaling Groups and Launch Templates
- Elastic Load Balancers (ALB, NLB, CLB)
- Lambda and serverless patterns

**Storage:**
- S3 (storage classes, lifecycle policies, encryption)
- EBS (volume types, snapshots, encryption)
- EFS and FSx file systems
- Storage Gateway

**Database:**
- RDS (Multi-AZ, read replicas, encryption)
- DynamoDB (partitioning, GSI, streams)
- ElastiCache (Redis vs Memcached)
- Database migration strategies

**Networking:**
- VPC (subnets, route tables, NACLs, security groups)
- Direct Connect and VPN
- Route 53 (routing policies, health checks)
- CloudFront and edge locations

**Security & Identity:**
- IAM (users, groups, roles, policies)
- AWS Organizations and SCPs
- GuardDuty, Inspector, Config
- Certificate Manager and encryption

### 8-Week Study Plan

**Week 1-2: Fundamentals**
- AWS Global Infrastructure
- Core compute services (EC2, Lambda)  
- Basic networking (VPC fundamentals)
- IAM and security basics

**Week 3-4: Storage & Databases**
- S3 deep dive and use cases
- EBS and EFS characteristics
- RDS vs DynamoDB selection criteria
- Caching strategies with ElastiCache

**Week 5-6: Architecture Patterns**
- Well-Architected Framework pillars
- High availability design patterns
- Disaster recovery strategies (RTO/RPO)
- Microservices and serverless architectures

**Week 7: Advanced Topics**
- Monitoring with CloudWatch
- Automation with CloudFormation
- Cost optimization strategies
- Migration patterns and tools

**Week 8: Practice & Review**
- Complete 3+ full practice exams
- Review incorrect answers thoroughly
- Focus on weak areas identified
- Final review of exam topics

### Hands-On Labs
1. **Multi-tier Web Application**
   - ALB + EC2 + RDS architecture
   - Auto Scaling and health checks
   - Security groups and NACLs

2. **Static Website with CloudFront**
   - S3 bucket configuration
   - CloudFront distribution setup
   - Route 53 DNS configuration

3. **Serverless Data Processing**
   - Lambda function triggers
   - DynamoDB data storage
   - API Gateway REST API

4. **Hybrid Cloud Connectivity**
   - VPN connection setup
   - Direct Connect simulation
   - Cross-region networking

### Key Architecture Patterns
- **3-Tier Architecture:** Web/App/Database layers
- **Microservices:** API Gateway + Lambda + DynamoDB
- **Data Lake:** S3 + Glue + Athena + QuickSight
- **Disaster Recovery:** Pilot Light, Warm Standby, Multi-Site
- **Caching Layers:** CloudFront, ElastiCache, DAX

### Exam Tips
- Eliminate obviously wrong answers first
- Look for keywords indicating specific services
- Consider cost-effectiveness in answers
- Think about operational overhead
- Pay attention to requirements (global, real-time, etc.)

### Post-Certification Goals
- Pursue AWS Solutions Architect Professional
- Gain hands-on experience with enterprise workloads
- Contribute to architecture decisions at work
- Consider specialty certifications (Security, DevOps)

### Study Schedule
- **Weekdays:** 1.5 hours (video + reading)
- **Saturdays:** 3 hours (hands-on labs)  
- **Sundays:** 2 hours (practice questions + review)
- **Total:** ~12 hours per week for 8 weeks"#.to_string(),
            parent_id: Some(learning_id),
            due_by: None,
        })?;

        Ok(())
    }

    fn mark_some_as_completed(&self, project_ids: &HashMap<String, i64>) -> anyhow::Result<()> {
        // Get some todos to mark as completed to show the app's completed view
        let todos = self.db.get_all_todos()?;
        
        // Mark about 30% of todos as completed with different completion dates
        let mut count = 0;
        let target_completed = todos.len() / 3;
        
        for todo in &todos {
            if count >= target_completed {
                break;
            }
            
            // Skip main project todos to keep hierarchy intact
            if project_ids.values().any(|&id| id == todo.id) {
                continue;
            }
            
            // Mark this todo as completed
            self.db.complete_todo(todo.id)?;
            count += 1;
        }

        Ok(())
    }
}