use window_from_scratch::win32::*;
use core::ptr::{null, null_mut};
use std::sync::mpsc::Receiver;

pub fn wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

pub unsafe extern "system" fn window_procedure(
    hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM,
  ) -> LRESULT {
    let mut rect = RECT::default();
    GetWindowRect(hWnd, &mut rect);
    match Msg {
      WM_MOUSEMOVE => {
        SetCursorPos(rect.right / 2, rect.bottom / 2);
      }
      WM_CLOSE => drop(DestroyWindow(hWnd)),
      WM_DESTROY => {
        let ptr = GetWindowLongPtrW(hWnd, GWLP_USERDATA) as *mut i32;
        Box::from_raw(ptr);
        println!("Cleaned up the box.");
        PostQuitMessage(0);
      },
      WM_PAINT => {
        let ptr = GetWindowLongPtrW(hWnd, GWLP_USERDATA) as *mut i32;
        println!("Current ptr: {}", *ptr);
        *ptr += 1;
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hWnd, &mut ps);
        let _success = FillRect(hdc, &ps.rcPaint, (COLOR_WINDOW + 3) as HBRUSH);
        EndPaint(hWnd, &ps);
      },
      WM_NCCREATE => {
        println!("NC Create");
        let createstruct: *mut CREATESTRUCTW = lParam as *mut _;
        if createstruct.is_null() {
          return 0;
        }
        let boxed_i32_ptr = (*createstruct).lpCreateParams;
        SetWindowLongPtrW(hWnd, GWLP_USERDATA, boxed_i32_ptr as LONG_PTR);
        return 1;
      },
      
      WM_CREATE => println!("Create"),
      _ => return DefWindowProcW(hWnd, Msg, wParam, lParam),
    }
    0
  }

fn main () 
{
    let hInstance = get_process_handle();
    let sample_window_class_wn = wide_null("Sample Window Class");

    let mut wc = WNDCLASSW::default();
    //let mut wc :WNDCLASSW =  unsafe { core::mem::zeroed() };
    wc.lpfnWndProc = Some(window_procedure);

    wc.hInstance = hInstance;
    wc.lpszClassName = sample_window_class_wn.as_ptr();
    //wc.hCursor = unsafe { LoadCursorW(hInstance, IDC_ARROW) };
    wc.hCursor = load_predefined_cursor(IDCursor::Arrow).unwrap();

    
    let atom = unsafe { register_class(&wc) }.unwrap_or_else(|()| {
      let last_error = unsafe { GetLastError() };
      panic!("Could not register the window class, error code: {}", last_error);
    });


    if atom == 0 {
        let last_error = unsafe { GetLastError() };
        panic!("Could not register the window class, error code: {}", last_error);
    }

    let sample_window_name_wn = wide_null("Sample Window Name");
   // in main
   let lparam: *mut i32 = Box::leak(Box::new(5_i32));
   let hwnd = unsafe {
     CreateWindowExW(
       0,
       sample_window_class_wn.as_ptr(),
       sample_window_name_wn.as_ptr(),
       WS_OVERLAPPEDWINDOW,
       CW_USEDEFAULT,
       CW_USEDEFAULT,
       CW_USEDEFAULT,
       CW_USEDEFAULT,
       null_mut(),
       null_mut(),
       hInstance,
       lparam.cast(),
     )
   };
    if hwnd.is_null() {
    panic!("Failed to create a window.");
    }

    let _previously_visible = unsafe { ShowWindow(hwnd, SW_SHOW) };

    let mut msg = MSG::default();
  loop {
    let message_return = unsafe { GetMessageW( &mut msg, null_mut(), 0, 0) };
    if message_return == 0 {
      break;
    } else if message_return == -1 {
      let last_error = unsafe { GetLastError() };
      panic!("Error with `GetMessageW`, error code: {}", last_error);
    }else {
        unsafe {
          TranslateMessage(&msg);
          DispatchMessageW(&msg);
        }
      }
  }
}