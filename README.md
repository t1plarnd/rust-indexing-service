# Ethereum Індексатор на Rust

Цей проєкт індексує транзакції Ethereum у базу даних PostgreSQL і надає REST API для їх отримання.

Проєкт використовує `alloy-rs`, `axum`, `sqlx` та `docker-compose`.

## Як запустити

Використовується Docker Compose для запуску всього стеку (API, База Даних, Nginx).

1.  **Вимоги:**
    * [Docker](https://www.docker.com/get-started)
    * [Docker Compose](https://docs.docker.com/compose/install/)

2.  **Клонуйте репозиторій:**
    ```bash
    git clone [https://github.com/your_username/your_project_name.git](https://github.com/your_username/your_project_name.git)
    cd your_project_name
    ```

3.  **Створіть файл `.env`:**
    Скопіюйте шаблон `env.example` і заповніть його вашими даними.
    ```bash
    cp .env.example .env
    ```
    Вам потрібно відредагувати `.env` та вставити ваш реальний `HTTP_INFURA_URL`.

4.  **Запустіть сервіс:**
    ```bash
    docker compose up --build
    ```

5.  **Все!** Сервіс тепер доступний за адресою `http://localhost`.

## API Ендпоінти

* `GET /transactions` - Отримати список останніх транзакцій.
* `GET /transactions?sender=0x...` - Фільтрувати за відправником.
* `GET /transactions?receiver=0x...` - Фільтрувати за отримувачем.
* `GET /transactions?participant=0x...` - Фільтрувати за учасником (відправник АБО отримувач).
* `GET /transactions/:hash` - Отримати одну транзакцію за її хешем.