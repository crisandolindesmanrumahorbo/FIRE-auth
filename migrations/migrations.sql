CREATE TABLE users (
	user_id SERIAL PRIMARY KEY,
	username VARCHAR(50) UNIQUE NOT NULL,
	password TEXT NOT NULL,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);


CREATE TABLE accounts (
  account_id SERIAL PRIMARY KEY,
  user_id INT NOT NULL,
  balance BIGINT NOT NULL,
  invested_value BIGINT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(user_id)
);

CREATE TABLE products (
  product_id SERIAL PRIMARY KEY,
  symbol VARCHAR(10) UNIQUE NOT NULL,
  name VARCHAR(100) UNIQUE NOT NULL,
  tags VARCHAR(100),
  last_updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);


CREATE INDEX idx_accounst_user ON accounts(user_id);

CREATE TABLE portfolios (
  portfolio_id SERIAL PRIMARY KEY,
  user_id INT NOT NULL,
  product_id INT NOT NULL,
  avg_price DECIMAL(10,2) NOT NULL,
  lot INT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(user_id),
  FOREIGN KEY (product_id) REFERENCES products(product_id),
);


/* TODO 
account balance
 */

