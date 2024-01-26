BAKERY_BIN_DIR="/usr/local/cargo/bin"
echo "Initializing env for '$(${BAKERY_BIN_DIR}/bakery --version)'"
PATH=${BAKERY_BIN_DIR}:${PATH}
# The BAKERY_CURRENT_BUILD_CONFIG will be set by
# bakery when initializing a workspace shell
alias build="${BAKERY_BIN_DIR}/bakery build -c ${BAKERY_CURRENT_BUILD_CONFIG}"
alias clean="${BAKERY_BIN_DIR}/bakery clean -c ${BAKERY_CURRENT_BUILD_CONFIG}"
alias deploy="${BAKERY_BIN_DIR}/bakery deploy -c ${BAKERY_CURRENT_BUILD_CONFIG}"
alias upload="${BAKERY_BIN_DIR}/bakery upload -c ${BAKERY_CURRENT_BUILD_CONFIG}"
