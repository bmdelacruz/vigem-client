typedef enum _VIGEM_ERRORS {
    VIGEM_ERROR_NONE = 0x20000000,
    VIGEM_ERROR_BUS_NOT_FOUND = 0xE0000001,
    VIGEM_ERROR_NO_FREE_SLOT = 0xE0000002,
    VIGEM_ERROR_INVALID_TARGET = 0xE0000003,
    VIGEM_ERROR_REMOVAL_FAILED = 0xE0000004,
    VIGEM_ERROR_ALREADY_CONNECTED = 0xE0000005,
    VIGEM_ERROR_TARGET_UNINITIALIZED = 0xE0000006,
    VIGEM_ERROR_TARGET_NOT_PLUGGED_IN = 0xE0000007,
    VIGEM_ERROR_BUS_VERSION_MISMATCH = 0xE0000008,
    VIGEM_ERROR_BUS_ACCESS_FAILED = 0xE0000009,
    VIGEM_ERROR_CALLBACK_ALREADY_REGISTERED = 0xE0000010,
    VIGEM_ERROR_CALLBACK_NOT_FOUND = 0xE0000011,
    VIGEM_ERROR_BUS_ALREADY_CONNECTED = 0xE0000012,
    VIGEM_ERROR_BUS_INVALID_HANDLE = 0xE0000013,
    VIGEM_ERROR_XUSB_USERINDEX_OUT_OF_RANGE = 0xE0000014,
    VIGEM_ERROR_INVALID_PARAMETER = 0xE0000015,
    VIGEM_ERROR_NOT_SUPPORTED = 0xE0000016
} VIGEM_ERROR;

typedef enum _XUSB_BUTTON {
    XUSB_GAMEPAD_DPAD_UP            = 0x0001,
    XUSB_GAMEPAD_DPAD_DOWN          = 0x0002,
    XUSB_GAMEPAD_DPAD_LEFT          = 0x0004,
    XUSB_GAMEPAD_DPAD_RIGHT         = 0x0008,
    XUSB_GAMEPAD_START              = 0x0010,
    XUSB_GAMEPAD_BACK               = 0x0020,
    XUSB_GAMEPAD_LEFT_THUMB         = 0x0040,
    XUSB_GAMEPAD_RIGHT_THUMB        = 0x0080,
    XUSB_GAMEPAD_LEFT_SHOULDER      = 0x0100,
    XUSB_GAMEPAD_RIGHT_SHOULDER     = 0x0200,
    XUSB_GAMEPAD_GUIDE              = 0x0400,
    XUSB_GAMEPAD_A                  = 0x1000,
    XUSB_GAMEPAD_B                  = 0x2000,
    XUSB_GAMEPAD_X                  = 0x4000,
    XUSB_GAMEPAD_Y                  = 0x8000
} XUSB_BUTTON;

typedef struct _XUSB_REPORT {
    unsigned short wButtons;
    char bLeftTrigger;
    char bRightTrigger;
    short sThumbLX;
    short sThumbLY;
    short sThumbRX;
    short sThumbRY;
} XUSB_REPORT;

typedef void* PVIGEM_CLIENT;
typedef void* PVIGEM_TARGET;
typedef void EVT_VIGEM_X360_NOTIFICATION(
    PVIGEM_CLIENT Client,
    PVIGEM_TARGET Target,
    unsigned char LargeMotor,
    unsigned char SmallMotor,
    unsigned char LedNumber,
    void* UserData
);
typedef EVT_VIGEM_X360_NOTIFICATION *PFN_VIGEM_X360_NOTIFICATION;

PVIGEM_CLIENT vigem_alloc(void);
void vigem_free(PVIGEM_CLIENT vigem);

VIGEM_ERROR vigem_connect(PVIGEM_CLIENT vigem);
void vigem_disconnect(PVIGEM_CLIENT vigem);

PVIGEM_TARGET vigem_target_x360_alloc(void);
void vigem_target_free(PVIGEM_TARGET target);

VIGEM_ERROR vigem_target_add(PVIGEM_CLIENT vigem, PVIGEM_TARGET target);
VIGEM_ERROR vigem_target_remove(PVIGEM_CLIENT vigem, PVIGEM_TARGET target);

VIGEM_ERROR vigem_target_x360_register_notification(PVIGEM_CLIENT vigem, PVIGEM_TARGET target, PFN_VIGEM_X360_NOTIFICATION notification, void* userData);
void* vigem_target_x360_unregister_notification(PVIGEM_TARGET target);

void vigem_target_lock_notification(PVIGEM_TARGET target);
void vigem_target_unlock_notification(PVIGEM_TARGET target);

VIGEM_ERROR vigem_target_x360_update(PVIGEM_CLIENT vigem, PVIGEM_TARGET target, XUSB_REPORT report);