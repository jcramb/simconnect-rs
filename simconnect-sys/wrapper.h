// inline Windows.h definitions

#define FALSE 0
#define CONST const
#define MAX_PATH 260
#define CALLBACK  __stdcall

typedef int BOOL;
typedef char CHAR;
typedef long LONG;
typedef void *PVOID;
typedef unsigned char BYTE;
typedef unsigned long DWORD;
typedef unsigned __int64 UINT64;

typedef PVOID HANDLE;
typedef HANDLE HWND;
typedef LONG HRESULT;
typedef /* __nullterminated */ CONST CHAR *LPCSTR;

typedef struct _GUID {
  unsigned long  Data1;
  unsigned short Data2;
  unsigned short Data3;
  unsigned char  Data4[8];
} GUID;

#include "sdk\\include\\SimConnect.h"