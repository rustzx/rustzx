SCRIPT_DIR=$(dirname "$0")
cd "${SCRIPT_DIR}"

OUT_DIR="${PWD}"
BUILD_DIR="${PWD}/build"
SRC_DIR="${PWD}/src"

ZMAKEBAS_VER="1.3"
ZMAKEBAS_TAR="${SRC_DIR}/third-party/zmakebas-${ZMAKEBAS_VER}.tar.gz"
ZMAKEBAS_DIR="${BUILD_DIR}/zmakebas-${ZMAKEBAS_VER}"
ZMAKEBAS_BIN="${ZMAKEBAS_DIR}/zmakebas"

LOG_INDENT_COUNT=0
COLOR_BLUE="\e[34m"
COLOR_RED="\e[31m"
COLOR_GREEN="\e[32m"
COLOR_NORMAL="\e[0m"

function print_log_indent {
    for i in $(seq $LOG_INDENT_COUNT); do
        echo -ne "\t"
    done
}

function log_indent {
    LOG_INDENT_COUNT=$(($LOG_INDENT_COUNT + 1))
}

function log_unindent {
    LOG_INDENT_COUNT=$(($LOG_INDENT_COUNT - 1))
}

function log_info {
    print_log_indent
    if [ $LOG_INDENT_COUNT -eq 0 ]; then
        echo -ne "${COLOR_BLUE}"
    fi
    echo "[INFO] $1"
    echo -ne "${COLOR_NORMAL}"
}

function log_error {
    print_log_indent
    echo -ne "${COLOR_RED}"
    echo "[ERROR] $1"
    echo -ne "${COLOR_NORMAL}"
}

function log_success {
    print_log_indent
    echo -ne "${COLOR_GREEN}"
    echo "[SUCCESS] $1"
    echo -ne "${COLOR_NORMAL}"
}

mkdir -p "${BUILD_DIR}"
mkdir -p "${BUILD_DIR}/loaders"
mkdir -p "${OUT_DIR}"

if [ ! -d "${ZMAKEBAS_DIR}" ]; then
    log_info "Building ZMAKEBAS..."
    log_indent
    OLD_PWD="${PWD}"
    tar -xzf "${ZMAKEBAS_TAR}" --directory "${BUILD_DIR}"
    log_info "Extracted"
    cd "${ZMAKEBAS_DIR}"
    make > /dev/null 2>&1
    if [ $? -ne 0 ]; then
        log_error "Failed to build ZMAKEBAS"
        exit 1
    fi
    log_success "Done"
    cd "${OLD_PWD}"
    log_unindent
fi

log_info "Building loaders..."
log_indent
${ZMAKEBAS_BIN} \
    -l \
    -a 10 \
    -n screen \
    -o "${BUILD_DIR}/loader_screen.tap" \
    "${SRC_DIR}/loader_screen.bas"
if [ $? -ne 0 ]; then
    log_error "Failed to build screen loader"
    exit 1
fi
log_success "Done"
log_unindent

log_info "Building simple_tape..."
log_indent
z88dk-appmake +zx \
    -b "${SRC_DIR}/rustzx.scr" \
    -o "${BUILD_DIR}/simple_tape_loaderless.tap" \
    --blockname screen \
    --org 16384 \
    --noloader
cat "${BUILD_DIR}/loader_screen.tap" \
    "${BUILD_DIR}/simple_tape_loaderless.tap" \
    > "${OUT_DIR}/simple_tape.tap"
log_success "Done"
log_unindent

function build_sna {
    local APP_NAME="$1"
    local ADDITIONAL_ARGS=""
    local EXT_PREFIX="48k"

    if [ "$2" = "128k" ]; then
        ADDITIONAL_ARGS="-Cz --128"
        local EXT_PREFIX="128k"
    fi

    log_info "Building ${APP_NAME} SNA (${EXT_PREFIX})..."
    log_indent
    zcc +zx \
        -lm \
        -o "${BUILD_DIR}/${APP_NAME}.${EXT_PREFIX}.o" \
        -create-app \
        -Cz --sna \
        -Cz -o \
        -Cz "${OUT_DIR}/${APP_NAME}.${EXT_PREFIX}.sna" \
        "${ADDITIONAL_ARGS}" \
        "${SRC_DIR}/${APP_NAME}.c"
    log_success "Done"
    log_unindent
}

build_sna sound 48k
build_sna sound 128k
