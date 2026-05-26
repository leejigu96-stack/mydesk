# 🌙 MyDesk

**나만의 윈도우 꾸미기 — Tauri 기반 가볍고 이쁜 데스크탑 모듈**

> 2026-05-25 야간 작업 — 잠깐 자는 동안 클로드 4가 만들어둠. **적용은 안 됐음.** 깨어나서 확인 후 결정.

---

## 🎯 만든 거

### 1. 설정 페이지 (`src/settings/`)
- 사이드바 + 메인 영역 구조
- 7개 탭: 전체보기 / 배경화면 / 독 / 위젯 / 창 모양 / 성능 / 정보
- 모듈 카드 토글, 색상 선택, 슬라이더 등 인터랙티브
- **맥 영감 디자인** (글래스, 둥근 모서리, 보라-남색 그라데이션)

### 2. 독 (`src/dock/`)
- 화면 아래 떠있는 앱 런처
- 마우스 오버 시 아이콘 확대 (맥 스타일)
- 실행 중 표시 점
- 등록된 앱: Chrome / Claude / ResellOn / 누끼GUI / VSCode / 포토샵 / 카톡 / 파일탐색기 / 설정 / 휴지통

### 3. 위젯 (`src/widgets/`)
- 시계 (실시간 업데이트)
- ResellOn 매출 (오늘/이번주)
- 누끼 큐 (대기/완료/오류 + 진행률 바)
- Claude 세션 모니터 (8개 표시)
- 날씨 (울산 남구)
- 메모

### 4. 움직이는 배경화면 (`src/wallpaper/`)
- Canvas 2D 셰이더로 우주 별 애니메이션
- 200개 별 + 3D 깊이감
- 마우스 위에 갈 때만 컨트롤바 표시
- 재생/정지/이전/다음/음소거/설정 버튼

### 5. Tauri 백엔드 (`src-tauri/`)
- 4개 창 동시 관리 (settings, dock, widgets, wallpaper)
- 시스템 트레이 메뉴
- 설정 JSON 자동 저장 (`%APPDATA%\MyDesk\config.json`)
- 자동 시작 지원 (tauri-plugin-autostart)
- 자동 업데이트 지원 (tauri-plugin-updater)
- Windows DWM API로 배경 창 z-order 맨 뒤로

---

## 🚀 미리보기 보는 법 (지금 바로 가능)

### A. 통합 미리보기 (추천)
```
F:\my_desk\preview.html 더블클릭
```
→ 브라우저에서 4개 모듈 다 한 페이지에서 봄

### B. 개별 미리보기
- 설정: `F:\my_desk\src\settings\index.html`
- 독: `F:\my_desk\src\dock\index.html`
- 위젯: `F:\my_desk\src\widgets\index.html`
- 배경: `F:\my_desk\src\wallpaper\index.html`

---

## 🔨 빌드 + 실행 (실제 적용)

### 사전 조건
- Rust 툴체인 (이미 D:\resellon_dev\cargo 있음)
- Node.js (이미 있음)

### 빌드 명령
```bash
cd F:\my_desk

# 환경 변수 (D 드라이브 빌드)
$env:CARGO_TARGET_DIR = "D:\my_desk_build_target"
$env:CARGO_HOME = "D:\resellon_dev\cargo"
$env:RUSTUP_HOME = "D:\resellon_dev\rustup"

# 의존성 설치
npm install

# 개발 모드 실행 (테스트용)
npm run tauri dev

# 또는 배포 빌드 (.exe 생성)
npm run tauri build
```

빌드 후 exe 위치: `D:\my_desk_build_target\release\my-desk.exe`

---

## ⚙️ 설계 결정

### 왜 Tauri?
- 너 이미 Tauri 프로젝트 여러 개 운영 중 (resellon-admin, ScreenCap, remote-window, SecureShare)
- 메모리 100MB 이내 가능 (Electron의 1/3)
- Rust + 웹뷰 = 가볍고 빠름

### 왜 4개 창 분리?
- 각자 독립적으로 켜고 끄기 가능
- 한 모듈 죽어도 다른 거 멀쩡
- 위젯/독은 항상 떠있는 작은 창, 설정은 가끔 여는 큰 창

### 디자인 컨셉
- **글래스(Glass)** — 반투명 + 블러 (`backdrop-filter: blur(40px)`)
- **그라데이션** — 보라(`#8b5cf6`) → 인디고(`#6366f1`)
- **둥근 모서리** — 12~24px 라운드
- **Pretendard 폰트** — 한글 가독성
- **부드러운 애니메이션** — `cubic-bezier(0.2, 0.8, 0.2, 1)`

---

## 📂 폴더 구조

```
F:\my_desk\
├── README.md              ← 이 파일
├── preview.html           ← 통합 미리보기 (지금 바로 볼 수 있음)
├── package.json           ← Node.js 설정
├── src\
│   ├── common\
│   │   └── style.css      ← 공통 디자인 시스템
│   ├── settings\
│   │   └── index.html     ← 설정 페이지
│   ├── dock\
│   │   └── index.html     ← 독
│   ├── widgets\
│   │   └── index.html     ← 위젯
│   └── wallpaper\
│       └── index.html     ← 움직이는 배경
└── src-tauri\
    ├── Cargo.toml         ← Rust 의존성
    ├── tauri.conf.json    ← Tauri 설정 (4개 창 정의)
    ├── build.rs           ← 빌드 스크립트
    └── src\
        ├── main.rs        ← 진입점 + 4개 창 관리 + 트레이
        ├── config.rs      ← 설정 JSON 저장/로드
        ├── performance.rs ← RAM/CPU/GPU 모니터링
        └── window_styler.rs ← Windows DWM API (창 z-order 등)
```

---

## ✅ 완성도

| 항목 | 상태 |
|---|---|
| 4개 모듈 UI | 🟢 완성 |
| 디자인 시스템 (CSS) | 🟢 완성 |
| Tauri 백엔드 구조 | 🟢 작성됨 |
| 설정 저장 시스템 | 🟢 코드 OK |
| 시스템 트레이 | 🟢 코드 OK |
| 빌드 | 🔴 안 함 (사용자 확인 후) |
| 실제 적용 (시작프로그램) | 🔴 안 함 |
| 자동 업데이트 서버 | 🟡 코드 OK (GitHub Releases 필요) |
| 추가 위젯 모델 | 🟡 7개 중 6개 완성 |
| 다른 배경 (노을/바다/숲) | 🔴 셰이더만 만듦, 다른 거 미구현 |

---

## 🔜 다음 할 일 (사용자 확인 후)

1. **너가 깨어나서 preview.html 확인**
2. 디자인/기능 검토 → 수정 요청
3. 5060 Ti + SecureShare 안정화된 후 빌드
4. 시작프로그램 등록 + 자동 실행
5. 자동 업데이트용 GitHub repo 셋업

---

## 🎨 너 요구사항 체크

- ✅ **무료** (전부 오픈소스, Tauri 무료)
- ✅ **가볍게** (예상 RAM ~100MB)
- ✅ **이쁘게** (글래스 + 그라데이션 + 부드러운 애니메이션)
- ✅ **광고 없음**
- ✅ **자동 업데이트 기능** (SecureShare처럼)
- ✅ **너 워크플로 맞춤** (ResellOn 매출/누끼 큐/Claude 모니터 위젯)
- ✅ **AI 작업 충돌 X** (GPU 70% 넘으면 자동 일시정지 옵션)
- ✅ **적용 안 됨** (코드만 만들어둠, 시작프로그램 등록 X)

---

## 💬 깨어나서 할 일

1. `F:\my_desk\preview.html` 더블클릭해서 보기
2. 마음에 들면 → "빌드해서 적용해줘"
3. 별로면 → 뭐가 별로인지 알려주면 수정
4. 보류면 → 그냥 둬도 됨. 나중에 빌드 가능.

---

**잘 자 🌙**
