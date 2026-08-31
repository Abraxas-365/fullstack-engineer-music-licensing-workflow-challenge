# AWS Deployment

Terraform for a production deployment: **ECS Fargate** for the backend, **S3 + CloudFront** for the frontend, **RDS Postgres**, all in a single VPC.

## Architecture

```
                 ┌────────────────────┐
Browser ───────▶ │  CloudFront (SPA)  │ ───▶ S3 (static assets, private via OAC)
   │             └────────────────────┘
   │ direct API calls (no CDN in front)
   ▼
┌────────────────────┐        ┌──────────────────────────┐
│  ALB (public subnet)│ ────▶ │ ECS Fargate (private subnet)│ ────▶ RDS Postgres (private subnet)
└────────────────────┘        └──────────────────────────┘
```

### Why the frontend isn't served *through* the backend's CDN path

The SPA is static (HTML/JS/CSS) — S3 + CloudFront serves it without running a container 24/7. CloudFront replaces the role `frontend/nginx.conf` plays in the Docker Compose setup (SPA fallback + static asset caching).

### Why CloudFront does NOT proxy `/api/*` to the backend


- The ALB is **public** and reachable directly (its own DNS name, or a `api.<domain>` record pointed at it).
- The SPA calls the ALB's domain directly for `/api/*` and `/api/licenses/events` (SSE) — no CDN hop.
- `CORS_ORIGIN` on the backend is set to the CloudFront domain so cross-origin requests from the SPA are allowed.

## Modules

| Module | Creates |
|--------|---------|
| `networking` | VPC, public/private subnets, IGW, single NAT gateway, security groups (ALB, ECS, RDS) |
| `database` | RDS Postgres (private, encrypted), credentials + connection string in Secrets Manager |
| `ecr` | ECR repository for the backend image (immutable tags, scan on push, keeps last 10 images) |
| `ecs-backend` | ALB (300s idle timeout for SSE), ECS Fargate service, task definition, autoscaling on CPU, CloudWatch logs |
| `frontend-static` | S3 bucket (private, OAC-only access), CloudFront distribution with SPA fallback (403/404 → index.html) |

## Prerequisites

- Terraform >= 1.5
- An AWS account + credentials configured (`aws configure` or env vars)
- Docker, to build and push the backend image to ECR

## Deploy

```bash
cd infra/terraform
terraform init
terraform apply
```

This provisions everything **except** the backend image — ECS will fail to start tasks until an image exists at `backend_ecr_repository_url:latest`. Build and push it:

```bash
# From the output of `terraform apply`:
REPO_URL=$(terraform output -raw backend_ecr_repository_url)

aws ecr get-login-password --region us-east-1 | docker login --username AWS --password-stdin "${REPO_URL%/*}"

docker build -t "$REPO_URL:latest" ../../backend
docker push "$REPO_URL:latest"

terraform apply
```

`terraform apply` on its own won't detect that a new image was pushed under the same `:latest` tag (Terraform only tracks the tag string, not the image digest), so force ECS to redeploy with the fresh image:

```bash
aws ecs update-service \
  --cluster "$(terraform output -raw ecs_cluster_name)" \
  --service "$(terraform output -raw ecs_service_name)" \
  --force-new-deployment
```

Then build and upload the frontend, pointing it at the ALB's URL:

```bash
BACKEND_URL=$(terraform output -raw backend_url)
cd ../../frontend
VITE_API_URL="$BACKEND_URL/api" npm run build
aws s3 sync dist/ "s3://$(cd ../infra/terraform && terraform output -raw frontend_bucket_name)" --delete

# Invalidate the CDN cache so the new build is served immediately
aws cloudfront create-invalidation \
  --distribution-id "$(cd ../infra/terraform && terraform output -raw cloudfront_distribution_id)" \
  --paths "/*"
```

## Outputs

After `terraform apply`, useful values are available via `terraform output`:

- `frontend_url` — CloudFront URL for the SPA
- `backend_url` — ALB URL for the API
- `backend_ecr_repository_url` — where to push backend images
- `frontend_bucket_name` / `cloudfront_distribution_id` — for frontend deploys
- `database_endpoint` — RDS address (not publicly reachable; for reference/debugging via a bastion or ECS Exec)

## Notes / things intentionally left out for a take-home scope

- **Custom domains + ACM certs**: supported via `frontend_domain_aliases` / `cloudfront_certificate_arn` (CloudFront, must be us-east-1) and `alb_certificate_arn` (ALB, same region as `aws_region`), but left unset by default — without them the ALB serves plain HTTP and CloudFront uses its default `*.cloudfront.net` certificate.
- **Remote state**: no S3/DynamoDB backend configured; state is local. For a real team setup, add a `backend "s3" {}` block in `versions.tf`.
- **CI/CD wiring** (build → push to ECR → `terraform apply` → sync S3 → invalidate CloudFront) is not automated here; the steps above are manual but map directly onto a GitHub Actions job.
- **Single NAT gateway** (not one per AZ) to keep cost down — acceptable for this workload, would add one per AZ for production HA.
