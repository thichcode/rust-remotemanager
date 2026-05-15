# 📖 User Guide — Hermes Remote Manager

> **Hermes Remote Manager** giúp bạn quản lý tất cả kết nối remote (SSH, RDP, Serial) tại một nơi — với terminal tích hợp, SFTP, port forwarding, và vault bảo mật.

---

## Mục lục

- [1. Cài đặt & Chạy lần đầu](#1-cài-đặt--chạy-lần-đầu)
- [2. Giao diện chính](#2-giao-diện-chính)
- [3. Quản lý kết nối](#3-quản-lý-kết-nối)
  - [3.1. Thêm kết nối mới](#31-thêm-kết-nối-mới)
  - [3.2. Chỉnh sửa kết nối](#32-chỉnh-sửa-kết-nối)
  - [3.3. Xóa kết nối](#33-xóa-kết-nối)
  - [3.4. Tìm kiếm](#34-tìm-kiếm)
  - [3.5. Thư mục (Folder)](#35-thư-mục-folder)
  - [3.6. Favorite](#36-favorite)
  - [3.7. Tags & Ghi chú](#37-tags--ghi-chú)
- [4. Terminal SSH](#4-terminal-ssh)
  - [4.1. Mở terminal](#41-mở-terminal)
  - [4.2. Commands khởi động (Startup Commands)](#42-commands-khởi-động-startup-commands)
  - [4.3. Resize terminal](#43-resize-terminal)
  - [4.4. Đóng session](#44-đóng-session)
- [5. Trình duyệt tệp SFTP](#5-trình-duyệt-tệp-sftp)
  - [5.1. Mở SFTP](#51-mở-sftp)
  - [5.2. Thao tác tệp](#52-thao-tác-tệp)
- [6. Port Forwarding (Tunnels)](#6-port-forwarding-tunnels)
  - [6.1. Tạo tunnel](#61-tạo-tunnel)
  - [6.2. Quản lý tunnel](#62-quản-lý-tunnel)
- [7. Vault — Lưu trữ mật khẩu an toàn](#7-vault--lưu-trữ-mật-khẩu-an-toàn)
  - [7.1. Khóa/mở khóa vault](#71-khóamở-khóa-vault)
  - [7.2. Thêm credential](#72-thêm-credential)
  - [7.3. Xóa credential](#73-xóa-credential)
- [8. Cài đặt (Settings)](#8-cài-đặt-settings)
- [9. Dữ liệu & Sao lưu](#9-dữ-liệu--sao-lưu)
- [10. Troubleshooting](#10-troubleshooting)

---

## 1. Cài đặt & Chạy lần đầu

### Yêu cầu hệ thống

| Thành phần | Phiên bản tối thiểu |
|-------------|---------------------|
| **Windows** | 10 (build 1903+) |
| **macOS** | 12 (Monterey+) |
| **Linux** | glibc 2.28+ |

### Cài đặt từ file portable

1. Tải file **`.zip`** portable từ [Releases](https://github.com/thichcode/rust-remotemanager/releases)
2. Giải nén vào thư mục bất kỳ, ví dụ: `C:\Hermes\`
3. Chạy **`hermes-remote-manager.exe`** (Windows) hoặc **`hermes-remote-manager`** (Linux/macOS)

> 💡 **Không cần cài đặt.** Toàn bộ dữ liệu nằm trong thư mục `%APPDATA%/hermes-remote-manager/` (Windows) hoặc `~/Library/Application Support/hermes-remote-manager/` (macOS) hoặc `~/.local/share/hermes-remote-manager/` (Linux).

### Cài đặt từ source

```bash
# Cần có Rust 1.75+ và Node.js 20+
git clone https://github.com/thichcode/rust-remotemanager.git
cd rust-remotemanager

npm install
npm run build
cargo tauri build
```

### Lần chạy đầu tiên

1. App khởi động → **Vault đang khóa** (mặc định)
2. Nhấn **🔒 Lock** → tạo mật khẩu vault
3. Tạo kết nối SSH đầu tiên (xem [3.1](#31-thêm-kết-nối-mới))

---

## 2. Giao diện chính

```
┌─────────────────────────────────────────────────────────┐
│  🔐 Hermes Remote Manager                    ⚙ Settings │
├──────────┬──────────────────────────────────────────────┤
│          │                                              │
│ 📁 All   │   Connection List                            │
│ 🔖 Favs  │   ┌──────────────────────────────────┐       │
│ 🏷 Tags  │   │ 🔌 SSH Server 1          [▶][📁] │       │
│          │   │ 🔌 Web Server             [▶][📁] │       │
│ Group:   │   │ 📁 Production              [📁]  │       │
│ [All]    │   │    ├── 🔌 DB Primary    [▶]     │       │
│ [Favs]   │   │    └── 🔌 DB Replica    [▶]     │       │
│          │   └──────────────────────────────────┘       │
│          │   Search connections...                       │
│          │                                              │
├──────────┴──────────────────────────────────────────────┤
│  Status: Connected to 3 sessions                       │
│  🔓 Vault: Locked                                       │
└─────────────────────────────────────────────────────────┘
```

| Vùng | Chức năng |
|------|-----------|
| **Thanh tiêu đề** | Tên app, nút Settings, nút Vault lock/unlock |
| **Sidebar trái** | Duyệt theo All / Favorites / Tags, hiển thị folder tree |
| **Vùng chính** | Danh sách kết nối, hỗ trợ tìm kiếm real-time |
| **Thanh trạng thái** | Số sessions đang hoạt động, trạng thái vault |

---

## 3. Quản lý kết nối

### 3.1. Thêm kết nối mới

1. Nhấn **"+"** (góc trên bên phải) hoặc **New Connection**
2. Điền thông tin:

| Trường | Bắt buộc | Mô tả |
|--------|----------|--------|
| **Name** | ✅ | Tên hiển thị (ví dụ: `Production DB`) |
| **Type** | ✅ | `SSH`, `RDP`, hoặc `Serial` |
| **Host** | ✅ | Địa chỉ IP hoặc hostname |
| **Port** | ✅ | Port (SSH mặc định: 22, RDP: 3389) |
| **Username** | ✅ | Tên đăng nhập |
| **Auth Type** | ✅ | `Password`, `Key`, hoặc `Keyboard Interactive` |
| **Credential** | — | Chọn credential đã lưu trong vault (tùy chọn) |
| **Tags** | — | Nhãn phân loại, cách nhau bằng dấu phẩy |
| **Notes** | — | Ghi chú riêng |

3. Nhấn **Save**

### 3.2. Chỉnh sửa kết nối

1. Nhấn chuột phải vào kết nối → **Edit**
2. Hoặc chọn kết nối → nhấn `F2`
3. Sửa thông tin → **Save**

### 3.3. Xóa kết nối

1. Chuột phải → **Delete**
2. Hoặc chọn → nhấn `Delete`
3. Xác nhận → dữ liệu bị xóa **vĩnh viễn** (không có recycle bin)

### 3.4. Tìm kiếm

- Gõ vào ô **Search** ở trên cùng
- Tìm kiếm theo: tên, host, tags, notes (không phân biệt hoa thường)
- Trống ô search → hiển thị tất cả

### 3.5. Thư mục (Folder)

Tổ chức kết nối theo nhóm:

| Thao tác | Cách thực hiện |
|----------|----------------|
| **Tạo folder** | Chuột phải sidebar → **New Folder** |
| **Chuyển kết nối vào folder** | Kéo thả kết nối vào folder |
| **Đổi tên folder** | Chuột phải → **Rename** |
| **Xóa folder** | Chuột phải → **Delete** (các kết nối bên trong **không** bị xóa, chỉ di chuyển ra ngoài) |
| **Sắp xếp** | Kéo thả để thay đổi thứ tự |

### 3.6. Favorite

- Nhấn ⭐ bên cạnh kết nối để đánh dấu **Favorite**
- Chuyển sang tab **Favorites** ở sidebar để xem danh sách yêu thích

### 3.7. Tags & Ghi chú

- **Tags**: dùng để phân loại (ví dụ: `production`, `staging`, `database`)
- Hiển thị trong sidebar dưới dạng **🏷 Tags** — nhấn vào tag để lọc
- **Notes**: thông tin bổ sung, chỉ hiển thị khi chỉnh sửa kết nối

---

## 4. Terminal SSH

### 4.1. Mở terminal

1. Chọn kết nối SSH trong danh sách
2. Nhấn **▶ (Connect)** hoặc nhấn đúp vào kết nối
3. Tab terminal mới mở ở phía dưới

### 4.2. Commands khởi động (Startup Commands)

Khi tạo/chỉnh sửa kết nối, bạn có thể đặt các lệnh sẽ tự động chạy sau khi kết nối:

```
# Ví dụ: tự động cd vào thư mục project và chạy docker compose
cd /opt/myapp
docker compose ps
```

- Mỗi lệnh trên **một dòng**
- Được thực thi tuần tự sau khi SSH handshake hoàn tất

### 4.3. Resize terminal

- Kéo cạnh cửa sổ terminal
- Hoặc dùng phím tắt: `Ctrl +` / `Ctrl -` (zoom)

### 4.4. Đóng session

- Nhấn nút **X** trên tab terminal
- Hoặc `Ctrl + D` trong terminal

---

## 5. Trình duyệt tệp SFTP

### 5.1. Mở SFTP

1. Đã có session SSH đang hoạt động
2. Nhấn nút **📁 SFTP** trên tab terminal hoặc menu **View → SFTP Browser**
3. Panel SFTP mở bên cạnh terminal

### 5.2. Thao tác tệp

| Thao tác | Cách thực hiện |
|----------|----------------|
| **Duyệt thư mục** | Nhấp đúp vào thư mục |
| **Upload tệp** | Kéo thả tệp từ máy tính vào panel SFTP |
| **Download tệp** | Chuột phải → **Download** |
| **Tạo thư mục mới** | Chuột phải → **New Directory** |
| **Xóa tệp/thư mục** | Chuột phải → **Delete** |
| **Đổi tên** | Chuột phải → **Rename** hoặc nhấn `F2` |
| **Xem thông tin** | Chuột phải → **Properties** (quyền, kích thước, ngày sửa) |

> ⚠️ **Lưu ý**: Panel SFTP chỉ hoạt động khi có session SSH đang kết nối.

---

## 6. Port Forwarding (Tunnels)

### 6.1. Tạo tunnel

Đây là tính năng chuyển tiếp cổng thông qua SSH tunnel (local forwarding):

1. Vào menu **Tunnels** hoặc sidebar **🔀 Tunnels**
2. Nhấn **+ New Tunnel**
3. Điền thông tin:

| Trường | Bắt buộc | Mô tả |
|--------|----------|--------|
| **Session** | ✅ | Chọn SSH session đang hoạt động |
| **Local Port** | ✅ | Port trên máy local (ví dụ: `5432`) |
| **Remote Host** | ✅ | Host đích phía remote (thường `localhost` hoặc `127.0.0.1`) |
| **Remote Port** | ✅ | Port đích phía remote (ví dụ: `5432` cho PostgreSQL) |

4. Nhấn **Create**

### 6.2. Quản lý tunnel

| Thao tác | Cách thực hiện |
|----------|----------------|
| **Bắt đầu tunnel** | Nhấn **▶ Start** |
| **Dừng tunnel** | Nhấn **⏹ Stop** |
| **Xóa tunnel** | Chuột phải → **Delete** |

### Ví dụ sử dụng phổ biến

```
Forward PostgreSQL từ server remote về local:
  Local Port:  5432
  Remote Host: 127.0.0.1
  Remote Port: 5432

→ Sau khi start, bạn có thể kết nối đến PostgreSQL trên máy local:
  psql -h 127.0.0.1 -p 5432 -U myuser mydb
```

---

## 7. Vault — Lưu trữ mật khẩu an toàn

Vault sử dụng **AES-256-GCM** để mã hóa và **Argon2id** để dẫn xuất khóa từ mật khẩu người dùng.

### 7.1. Khóa/mở khóa vault

| Hành động | Thao tác |
|-----------|----------|
| **Mở khóa** | Nhấn **🔓 Unlock** → nhập mật khẩu vault |
| **Khóa lại** | Nhấn **🔒 Lock** trên thanh tiêu đề |

> ⚠️ Nếu quên mật khẩu vault, **không có cách nào khôi phục**. Mật khẩu không được lưu hay gửi đi bất kỳ đâu.

### 7.2. Thêm credential

1. Mở vault (unlock)
2. Vào sidebar **🔑 Credentials**
3. Nhấn **+ Add Credential**
4. Điền:

| Trường | Bắt buộc | Mô tả |
|--------|----------|--------|
| **Name** | ✅ | Tên (ví dụ: `Production DB Password`) |
| **Auth Type** | ✅ | `Password` hoặc `SSH Key` |
| **Username** | — | Tên đăng nhập |
| **Password** | — | Mật khẩu (nếu auth type = Password) |
| **Private Key** | — | Nội dung private key (nếu auth type = SSH Key) |

5. Nhấn **Save** → credential được mã hóa và lưu vào SQLite

### 7.3. Xóa credential

1. Chọn credential trong danh sách
2. Nhấn **Delete** hoặc chuột phải → **Delete**
3. Xác nhận xóa

### Sử dụng credential khi kết nối

Khi tạo/chỉnh sửa kết nối SSH:
1. Chọn **Auth Type: Key** hoặc **Password**
2. Ở trường **Credential**, chọn credential đã lưu trong vault
3. App sẽ tự điền thông tin khi kết nối

---

## 8. Cài đặt (Settings)

Vào menu **⚙ Settings** để cấu hình:

| Cài đặt | Mặc định | Mô tả |
|---------|----------|--------|
| **Theme** | System | Giao diện sáng/tối theo hệ thống |
| **Font Size** | 14px | Cỡ chữ trong terminal |
| **Keepalive Interval** | 60s | Gửi keepalive mỗi N giây để duy trì kết nối SSH |
| **Auto-reconnect** | On | Tự động kết nối lại khi mất kết nối |

---

## 9. Dữ liệu & Sao lưu

### Vị trí lưu dữ liệu

| Hệ điều hành | Đường dẫn |
|--------------|-----------|
| **Windows** | `%APPDATA%\hermes-remote-manager\` |
| **macOS** | `~/Library/Application Support/hermes-remote-manager/` |
| **Linux** | `~/.local/share/hermes-remote-manager/` |

### File quan trọng

| File | Mô tả |
|------|--------|
| `hermes.db` | Cơ sở dữ liệu SQLite chứa tất cả kết nối, credentials, cài đặt |
| `vault.key` | Khóa mã hóa vault (được dẫn xuất từ mật khẩu, **KHÔNG** lưu trực tiếp) |

### Sao lưu thủ công

1. Đóng ứng dụng
2. Copy toàn bộ thư mục data bên trên sang nơi an toàn
3. Để khôi phục: cài app mới → thay thế thư mục data

> 💡 **Best practice**: Sao lưu `hermes.db` định kỳ, đặc biệt sau khi thêm nhiều kết nối hoặc credential mới.

---

## 10. Troubleshooting

### SSH Connection refused

```
❌ Lỗi: "Connection refused"
```

**Nguyên nhân & cách fix:**
- Kiểm tra SSH đã được bật trên server: `sudo systemctl status sshd`
- Kiểm tra firewall: `sudo ufw allow 22`
- Kiểm tra port đúng (mặc định SSH là 22)

### SSH Authentication failed

```
❌ Lỗi: "Authentication failed"
```

**Nguyên nhân & cách fix:**
- Kiểm tra username đúng
- Với key auth: đảm bảo public key đã thêm vào `~/.ssh/authorized_keys` trên server
- Với password: kiểm tra mật khẩu trong vault (có thể đã expired hoặc bị thay đổi)
- Kiểm tra `PasswordAuthentication yes` trong `/etc/ssh/sshd_config`

### SFTP không hiển thị tệp

```
❌ Lỗi: Panel SFTP trống hoặc báo lỗi
```

**Nguyên nhân & cách fix:**
- Đảm bảo SSH session đang **kết nối thành công** (SFTP cần session SSH hoạt động)
- Kiểm tra user có quyền đọc thư mục trên server
- Thử reconnect SSH session rồi mở lại SFTP

### Vault không mở khóa được

```
❌ Lỗi: "Incorrect vault password"
```

**Nguyên nhân & cách fix:**
- Kiểm tra Caps Lock
- Mật khẩu vault **phân biệt hoa thường**
- Nếu quên mật khẩu: **không thể khôi phục** — cần xóa data và tạo lại

### Terminal hiển thị loạn ký tự

```
❌ Ký tự hiển thị bị lỗi hoặc chồng chéo
```

**Nguyên nhân & cách fix:**
- Thay đổi font size trong **Settings → Font Size**
- Kiểm tra encoding: server cần hỗ trợ UTF-8 (`export LANG=en_US.UTF-8`)
- Resize terminal window

### Tunnel không hoạt động

```
❌ Kết nối đến local port bị từ chối
```

**Nguyên nhân & cách fix:**
- Đảm bảo tunnel đã được **Start** (nhấn ▶)
- Đảm bảo SSH session đang hoạt động
- Kiểm tra remote host/port đích có đúng service đang chạy
- Kiểm tra ứng dụng local (ví dụ PostgreSQL) đã **listen trên 127.0.0.1**

### App crash khi khởi động

```
❌ Ứng dụng đóng ngay sau khi mở
```

**Nguyên nhân & cách fix:**
- Xóa thư mục data và chạy lại (sẽ mất toàn bộ dữ liệu):
  ```bash
  # Windows
  rmdir /s /q %APPDATA%\hermes-remote-manager

  # macOS
  rm -rf ~/Library/Application\ Support/hermes-remote-manager

  # Linux
  rm -rf ~/.local/share/hermes-remote-manager
  ```
- Đảm bảo phiên bản app tương thích với hệ điều hành

---

## Phím tắt nhanh

| Phím tắt | Chức năng |
|----------|-----------|
| `Ctrl + N` | Tạo kết nối mới |
| `F2` | Chỉnh sửa kết nối đang chọn |
| `Delete` | Xóa kết nối đang chọn |
| `Ctrl + F` | Tìm kiếm |
| `Ctrl + Enter` | Kết nối đến server đang chọn |
| `Ctrl + W` | Đóng tab terminal/SFTP hiện tại |
| `Ctrl + Shift + L` | Khóa vault |
| `F5` | Refresh danh sách |

---

## 💡 Tips & Tricks

1. **Sử dụng Tags hiệu quả**: Gắn tag `production`, `staging`, `dev` để lọc nhanh
2. **Startup Commands**: Đặt các lệnh như `tmux attach` hoặc `source ~/.bashrc` để tự động setup môi trường
3. **Credential reuse**: Một credential có thể dùng cho nhiều kết nối khác nhau
4. **SFTP drag & drop**: Kéo thả tệp trực tiếp giữa máy tính và server
5. **Multiple sessions**: Mở nhiều tab terminal đồng thời để quản lý nhiều server

---

## 🆘 Hỗ trợ

Nếu gặp vấn đề không được giải quyết trong hướng dẫn này:

1. Kiểm tra log ứng dụng: `%APPDATA%\hermes-remote-manager\*.log`
2. Mở issue trên GitHub: https://github.com/thichcode/rust-remotemanager/issues
3. Bao gồm thông tin:
   - Phiên bản app (xem **About** trong menu)
   - Hệ điều hành và phiên bản
   - Mô tả chi tiết lỗi
   - Screenshot nếu có

---

*Cập nhật lần cuối: 2026-05*