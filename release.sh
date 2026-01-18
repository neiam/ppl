#!/bin/bash

# NEIAM Release Script
# This script helps create and manage releases for the NEIAM projects

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Function to get current version from Cargo.toml
get_current_version() {
    grep '^version = ' Cargo.toml | cut -d'"' -f2
}

# Function to validate version format
validate_version() {
    local version=$1
    if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$ ]]; then
        print_status $RED "Invalid version format: $version"
        print_status $YELLOW "Version should be in format: X.Y.Z or X.Y.Z-suffix"
        return 1
    fi
}

# Function to update version in Cargo.toml
update_version() {
    local new_version=$1
    print_status $BLUE "Updating version to $new_version in Cargo.toml..."
    sed -i "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
    print_status $GREEN "Version updated successfully!"
}

# Function to create git tag
create_tag() {
    local version=$1
    local tag_name="v$version"
    
    print_status $BLUE "Creating git tag $tag_name..."
    
    # Check if tag already exists
    if git tag -l | grep -q "^$tag_name$"; then
        print_status $RED "Tag $tag_name already exists!"
        return 1
    fi
    
    # Create annotated tag
    git tag -a "$tag_name" -m "Release $tag_name"
    print_status $GREEN "Tag $tag_name created successfully!"
}

# Function to show release preparation checklist
show_checklist() {
    print_status $BLUE "Pre-release Checklist:"
    echo
    echo "  [ ] All tests are passing"
    echo "  [ ] Documentation is up to date"
    echo "  [ ] CHANGELOG.md is updated (if exists)"
    echo "  [ ] All changes are committed"
    echo "  [ ] Working directory is clean"
    echo "  [ ] On the correct branch (usually master/main)"
    echo
}

# Function to check working directory
check_working_directory() {
    if [[ -n $(git status --porcelain) ]]; then
        print_status $RED "Working directory is not clean!"
        print_status $YELLOW "Please commit or stash your changes first."
        git status --short
        return 1
    fi
}

# Function to check current branch
check_branch() {
    local current_branch=$(git branch --show-current)
    if [[ $current_branch != "master" && $current_branch != "main" ]]; then
        print_status $YELLOW "Warning: You are on branch '$current_branch', not on master/main"
        read -p "Do you want to continue? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_status $RED "Release cancelled."
            return 1
        fi
    fi
}

# Function to run tests
run_tests() {
    print_status $BLUE "Running tests..."
    if cargo test; then
        print_status $GREEN "All tests passed!"
    else
        print_status $RED "Tests failed! Please fix issues before releasing."
        return 1
    fi
}

# Function to build project
build_project() {
    print_status $BLUE "Building project..."
    if cargo build --release; then
        print_status $GREEN "Build successful!"
    else
        print_status $RED "Build failed! Please fix issues before releasing."
        return 1
    fi
}

# Function to push release
push_release() {
    local version=$1
    local tag_name="v$version"
    
    print_status $BLUE "Pushing release to GitHub..."
    
    # Push commits
    git push origin
    
    # Push tag
    git push origin "$tag_name"
    
    print_status $GREEN "Release pushed successfully!"
    print_status $BLUE "GitHub Actions will now build and create the release."
    print_status $BLUE "Check the Actions tab in your GitHub repository for progress."
}

# Function to show help
show_help() {
    cat << EOF
NEIAM Release Script

Usage: $0 <command> [options]

Commands:
  current               Show current version
  prepare <version>     Prepare a new release (update version, create tag)
  patch                 Increment patch version and prepare release
  minor                 Increment minor version and prepare release
  major                 Increment major version and prepare release
  push <version>        Push release tag to trigger GitHub Actions
  checklist            Show pre-release checklist
  help                  Show this help message

Options:
  --dry-run            Show what would be done without making changes
  --no-tests           Skip running tests
  --no-build           Skip building project

Examples:
  $0 current                    # Show current version
  $0 prepare 1.2.3              # Prepare version 1.2.3
  $0 patch                      # Increment patch version (1.2.3 -> 1.2.4)
  $0 minor                      # Increment minor version (1.2.3 -> 1.3.0)
  $0 major                      # Increment major version (1.2.3 -> 2.0.0)
  $0 push 1.2.3                 # Push v1.2.3 tag to trigger release
  $0 prepare 1.2.3 --dry-run    # Show what would be done

Release Process:
  1. Run '$0 checklist' to review pre-release requirements
  2. Run '$0 prepare <version>' to update version and create tag
  3. Run '$0 push <version>' to push and trigger GitHub Actions release
  
  Or use shortcuts:
  - '$0 patch' for patch releases
  - '$0 minor' for minor releases
  - '$0 major' for major releases
EOF
}

# Function to increment version
increment_version() {
    local current_version=$(get_current_version)
    local increment_type=$1
    
    IFS='.' read -r major minor patch <<< "$current_version"
    
    case $increment_type in
        "patch")
            patch=$((patch + 1))
            ;;
        "minor")
            minor=$((minor + 1))
            patch=0
            ;;
        "major")
            major=$((major + 1))
            minor=0
            patch=0
            ;;
    esac
    
    echo "$major.$minor.$patch"
}

# Main function
main() {
    local command=${1:-help}
    local dry_run=false
    local no_tests=false
    local no_build=false
    
    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                dry_run=true
                shift
                ;;
            --no-tests)
                no_tests=true
                shift
                ;;
            --no-build)
                no_build=true
                shift
                ;;
            -*)
                print_status $RED "Unknown option: $1"
                show_help
                exit 1
                ;;
            *)
                break
                ;;
        esac
    done
    
    case $command in
        "current")
            local current_version=$(get_current_version)
            print_status $BLUE "Current version: $current_version"
            ;;
        "prepare")
            local version=${2:-}
            if [[ -z $version ]]; then
                print_status $RED "Version is required for prepare command"
                show_help
                exit 1
            fi
            
            validate_version "$version"
            
            if [[ $dry_run == true ]]; then
                print_status $YELLOW "DRY RUN: Would update version to $version and create tag v$version"
                exit 0
            fi
            
            check_working_directory
            check_branch
            
            if [[ $no_tests == false ]]; then
                run_tests
            fi
            
            if [[ $no_build == false ]]; then
                build_project
            fi
            
            update_version "$version"
            
            # Commit version change
            git add Cargo.toml
            git commit -m "Bump version to $version"
            
            create_tag "$version"
            
            print_status $GREEN "Release $version prepared successfully!"
            print_status $BLUE "Run '$0 push $version' to push and trigger GitHub Actions release"
            ;;
        "patch"|"minor"|"major")
            local new_version=$(increment_version "$command")
            print_status $BLUE "Incrementing $command version to $new_version"
            
            if [[ $dry_run == true ]]; then
                print_status $YELLOW "DRY RUN: Would increment $command version to $new_version"
                exit 0
            fi
            
            # Call prepare with the new version
            main "prepare" "$new_version"
            ;;
        "push")
            local version=${2:-}
            if [[ -z $version ]]; then
                print_status $RED "Version is required for push command"
                show_help
                exit 1
            fi
            
            validate_version "$version"
            
            if [[ $dry_run == true ]]; then
                print_status $YELLOW "DRY RUN: Would push tag v$version"
                exit 0
            fi
            
            push_release "$version"
            ;;
        "checklist")
            show_checklist
            ;;
        "help"|*)
            show_help
            ;;
    esac
}

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    print_status $RED "This script must be run from within a git repository"
    exit 1
fi

# Check if Cargo.toml exists
if [[ ! -f Cargo.toml ]]; then
    print_status $RED "Cargo.toml not found. This script must be run from the project root."
    exit 1
fi

main "$@"
