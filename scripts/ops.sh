#!/usr/bin/env bash
# Archives Operations Framework
# Source this file, don't execute it: source scripts/ops.sh

# Detect if being sourced or executed
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "Error: This script should be sourced, not executed."
    echo "Usage: source scripts/ops.sh"
    exit 1
fi

# Project root detection
ARCHIVES_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export ARCHIVES_ROOT

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ============================================================================
# Core Functions
# ============================================================================

ops() {
    local cmd="${1:-help}"
    shift 2>/dev/null || true

    case "$cmd" in
        build)      _ops_build "$@" ;;
        start)      _ops_start "$@" ;;
        stop)       _ops_stop "$@" ;;
        restart)    _ops_stop "$@" && _ops_start "$@" ;;
        dev)        _ops_dev "$@" ;;
        test)       _ops_test "$@" ;;
        lint)       _ops_lint "$@" ;;
        fmt)        _ops_fmt "$@" ;;
        check)      _ops_check "$@" ;;
        status)     _ops_status "$@" ;;
        logs)       _ops_logs "$@" ;;
        clean)      _ops_clean "$@" ;;
        help|*)     _ops_help ;;
    esac
}

# ============================================================================
# Command Implementations
# ============================================================================

_ops_build() {
    local target="${1:-all}"
    echo -e "${BLUE}Building Archives${NC} ($target)..."

    cd "$ARCHIVES_ROOT"

    case "$target" in
        all)
            cargo build --workspace
            ;;
        release)
            cargo build --workspace --release
            ;;
        api)
            cargo build -p archives-api
            ;;
        mcp)
            cargo build -p archives-mcp
            ;;
        cli)
            cargo build -p archives-cli
            ;;
        docker)
            docker compose build
            ;;
        *)
            echo -e "${RED}Unknown build target:${NC} $target"
            return 1
            ;;
    esac

    echo -e "${GREEN}Build complete${NC}"
}

_ops_start() {
    local service="${1:-infra}"
    echo -e "${BLUE}Starting Archives${NC} ($service)..."

    cd "$ARCHIVES_ROOT"

    case "$service" in
        infra)
            # Start ClickHouse and OTEL Collector
            docker compose up -d clickhouse otel-collector
            echo -e "${GREEN}Infrastructure started${NC}"
            echo "ClickHouse: http://localhost:8123"
            echo "OTLP gRPC:  localhost:4317"
            echo "OTLP HTTP:  http://localhost:4318"
            ;;
        api)
            cargo run -p archives-api &
            ;;
        mcp)
            cargo run -p archives-mcp &
            ;;
        all)
            docker compose up -d
            ;;
        *)
            docker compose up -d "$service"
            ;;
    esac
}

_ops_stop() {
    local service="${1:-all}"
    echo -e "${BLUE}Stopping Archives${NC} ($service)..."

    cd "$ARCHIVES_ROOT"

    case "$service" in
        all)
            docker compose down
            pkill -f "archives-api" 2>/dev/null || true
            pkill -f "archives-mcp" 2>/dev/null || true
            ;;
        infra)
            docker compose down
            ;;
        *)
            docker compose stop "$service"
            ;;
    esac

    echo -e "${GREEN}Stopped${NC}"
}

_ops_dev() {
    local service="${1:-api}"
    echo -e "${BLUE}Starting Archives dev mode${NC} ($service)..."

    cd "$ARCHIVES_ROOT"

    # Ensure infrastructure is running
    docker compose up -d clickhouse otel-collector

    case "$service" in
        api)
            RUST_LOG=archives_api=debug cargo run -p archives-api
            ;;
        mcp)
            RUST_LOG=archives_mcp=debug cargo run -p archives-mcp
            ;;
        *)
            echo -e "${RED}Unknown service:${NC} $service (use: api, mcp)"
            return 1
            ;;
    esac
}

_ops_test() {
    local scope="${1:-all}"
    echo -e "${BLUE}Running Archives tests${NC} ($scope)..."

    cd "$ARCHIVES_ROOT"

    case "$scope" in
        all)
            cargo test --workspace
            ;;
        unit)
            cargo test --workspace --lib
            ;;
        integration)
            cargo test --workspace --test '*'
            ;;
        *)
            cargo test -p "archives-$scope"
            ;;
    esac
}

_ops_lint() {
    echo -e "${BLUE}Running clippy lints${NC}..."
    cd "$ARCHIVES_ROOT"
    cargo clippy --workspace --all-targets -- -D warnings
    echo -e "${GREEN}Lint complete${NC}"
}

_ops_fmt() {
    local mode="${1:-check}"
    cd "$ARCHIVES_ROOT"

    case "$mode" in
        check)
            echo -e "${BLUE}Checking formatting${NC}..."
            cargo fmt --all -- --check
            ;;
        fix)
            echo -e "${BLUE}Formatting code${NC}..."
            cargo fmt --all
            ;;
        *)
            echo -e "${RED}Unknown format mode:${NC} $mode (use: check, fix)"
            return 1
            ;;
    esac
    echo -e "${GREEN}Format complete${NC}"
}

_ops_check() {
    echo -e "${BLUE}Running all checks (fmt, lint, test)${NC}..."
    cd "$ARCHIVES_ROOT"

    echo -e "\n${YELLOW}Step 1/3: Format check${NC}"
    cargo fmt --all -- --check || { echo -e "${RED}Format check failed${NC}"; return 1; }

    echo -e "\n${YELLOW}Step 2/3: Clippy lint${NC}"
    cargo clippy --workspace --all-targets -- -D warnings || { echo -e "${RED}Lint check failed${NC}"; return 1; }

    echo -e "\n${YELLOW}Step 3/3: Tests${NC}"
    cargo test --workspace || { echo -e "${RED}Tests failed${NC}"; return 1; }

    echo -e "\n${GREEN}All checks passed!${NC}"
}

_ops_status() {
    echo -e "${BLUE}Archives Status${NC}"
    echo "================"

    cd "$ARCHIVES_ROOT"

    # Docker services
    echo -e "\n${YELLOW}Docker Services:${NC}"
    docker compose ps --format "table {{.Name}}\t{{.Status}}\t{{.Ports}}" 2>/dev/null || echo "Docker not running"

    # Check ClickHouse
    echo -e "\n${YELLOW}ClickHouse:${NC}"
    if curl -s "http://localhost:8123/ping" > /dev/null 2>&1; then
        echo -e "  ${GREEN}● Connected${NC}"
        # Get stats
        curl -s "http://localhost:8123" --data "SELECT count() FROM system.parts WHERE active" 2>/dev/null && echo " active parts" || true
    else
        echo -e "  ${RED}● Not connected${NC}"
    fi

    # Check OTEL Collector
    echo -e "\n${YELLOW}OTEL Collector:${NC}"
    if curl -s "http://localhost:13133/" > /dev/null 2>&1; then
        echo -e "  ${GREEN}● Healthy${NC}"
    else
        echo -e "  ${RED}● Not running${NC}"
    fi

    # Check API
    echo -e "\n${YELLOW}Archives API:${NC}"
    if curl -s "http://localhost:8080/health" > /dev/null 2>&1; then
        echo -e "  ${GREEN}● Running${NC}"
    else
        echo -e "  ${YELLOW}● Not running${NC}"
    fi

    # Check MCP
    echo -e "\n${YELLOW}Archives MCP:${NC}"
    if curl -s "http://localhost:8081/health" > /dev/null 2>&1; then
        echo -e "  ${GREEN}● Running${NC}"
    else
        echo -e "  ${YELLOW}● Not running${NC}"
    fi
}

_ops_logs() {
    local service="${1:-otel-collector}"
    local follow="${2:--f}"

    cd "$ARCHIVES_ROOT"
    docker compose logs "$follow" "$service"
}

_ops_clean() {
    echo -e "${BLUE}Cleaning Archives${NC}..."

    cd "$ARCHIVES_ROOT"

    # Stop all services
    docker compose down -v 2>/dev/null || true

    # Clean Rust build artifacts
    cargo clean

    # Remove local volumes
    rm -rf logs/*.log 2>/dev/null || true

    echo -e "${GREEN}Clean complete${NC}"
}

_ops_help() {
    cat << 'EOF'
Archives Operations Framework

Usage: ops <command> [args]

Commands:
  build [target]     Build the project
                     Targets: all, release, api, mcp, cli, docker

  start [service]    Start services
                     Services: infra (default), api, mcp, all

  stop [service]     Stop services
                     Services: all (default), infra, api, mcp

  restart [service]  Restart services

  dev [service]      Run in development mode with hot reload
                     Services: api (default), mcp

  test [scope]       Run tests
                     Scopes: all, unit, integration, common, api, mcp, cli

  lint               Run clippy lints

  fmt [mode]         Check or fix code formatting
                     Modes: check (default), fix

  check              Run all checks (fmt, lint, test)

  status             Show status of all services

  logs [service]     Tail service logs

  clean              Clean build artifacts and volumes

  help               Show this help message

Examples:
  ops start infra    # Start ClickHouse and OTEL Collector
  ops dev api        # Run API server in dev mode
  ops logs otel-collector
  ops test common
  ops lint           # Run clippy
  ops fmt fix        # Format code
  ops check          # Run all CI checks
EOF
}

# Print welcome message
echo -e "${GREEN}Archives ops loaded.${NC} Type 'ops help' for commands."
