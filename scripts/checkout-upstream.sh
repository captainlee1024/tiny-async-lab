#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
checkout_root="${repo_root}/upstream/checkouts"

fail() {
    echo "错误：$*" >&2
    exit 1
}

prepare_checkout() {
    local name="$1"
    local directory="$2"
    local remote_url="$3"
    local tag="$4"
    local expected_commit="$5"
    local path="${checkout_root}/${directory}"
    local actual_remote
    local checkout_path
    local repository_root
    local tag_commit
    local head_commit
    local status

    if [[ ! -e "${path}" ]]; then
        mkdir -p "${path}"
        git -C "${path}" init --quiet
        git -C "${path}" remote add origin "${remote_url}"
    elif [[ ! -d "${path}" ]]; then
        fail "${path} 已存在，但不是目录。"
    fi

    if ! repository_root="$(git -C "${path}" rev-parse --show-toplevel 2>/dev/null)"; then
        fail "${path} 不是独立的 Git checkout；请移走该目录后重试。"
    fi

    checkout_path="$(cd "${path}" && pwd -P)"
    repository_root="$(cd "${repository_root}" && pwd -P)"
    if [[ "${checkout_path}" != "${repository_root}" ]]; then
        fail "${path} 不是独立的 Git checkout；请移走该目录后重试。"
    fi

    if ! actual_remote="$(git -C "${path}" remote get-url origin 2>/dev/null)"; then
        fail "${name} checkout 缺少 origin remote；期望 ${remote_url}。"
    fi
    if [[ "${actual_remote}" != "${remote_url}" ]]; then
        fail "${name} checkout 的 origin 为 ${actual_remote}；期望 ${remote_url}。"
    fi

    status="$(git -C "${path}" status --porcelain --untracked-files=normal)"
    if [[ -n "${status}" ]]; then
        fail "${name} checkout 含有未提交变更；请先运行 git -C ${path} status --short 并自行处理。"
    fi

    if [[ "$(git -C "${path}" rev-parse --is-shallow-repository)" == "true" ]]; then
        echo "正在补全 ${name} 的 Git 历史……"
        git -C "${path}" fetch --quiet --unshallow --prune --tags origin
    fi

    if ! tag_commit="$(git -C "${path}" rev-parse --verify "refs/tags/${tag}^{commit}" 2>/dev/null)"; then
        echo "正在从 ${remote_url} 获取 ${name} 的完整 Git 历史……"
        git -C "${path}" fetch --quiet --prune --tags origin
        if ! tag_commit="$(git -C "${path}" rev-parse --verify "refs/tags/${tag}^{commit}" 2>/dev/null)"; then
            fail "${name} 的 tag ${tag} 不存在。"
        fi
    fi

    if [[ "${tag_commit}" != "${expected_commit}" ]]; then
        fail "${name} 的 tag ${tag} 指向 ${tag_commit}；期望 ${expected_commit}。"
    fi

    head_commit="$(git -C "${path}" rev-parse --verify HEAD 2>/dev/null || true)"
    if [[ "${head_commit}" != "${expected_commit}" ]]; then
        git -C "${path}" checkout --detach --quiet "${expected_commit}"
    fi

    head_commit="$(git -C "${path}" rev-parse --verify HEAD)"
    status="$(git -C "${path}" status --porcelain --untracked-files=normal)"
    if [[ "${head_commit}" != "${expected_commit}" || -n "${status}" ]]; then
        fail "${name} checkout 未达到预期的干净固定状态。"
    fi

    echo "${name}: ${tag} (${expected_commit})"
}

mkdir -p "${checkout_root}"

prepare_checkout \
    "Tokio" \
    "tokio" \
    "https://github.com/tokio-rs/tokio" \
    "tokio-1.52.3" \
    "d87569164fb61145e79e7ffe0b25783569cc8f93"

prepare_checkout \
    "Mio" \
    "mio" \
    "https://github.com/tokio-rs/mio" \
    "v1.2.0" \
    "ce39a6be2cc739165daaeb10cce609b9b77242ac"
