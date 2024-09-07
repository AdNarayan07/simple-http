pub fn generate_html(title: String, body: String) -> String {
    format!(r#"
    <html>
    <head>
        <title>{title}</title>
        <meta charset="utf-8">
        <style>
        body {{
            margin: 20px;
            font-family: system-ui;
        }}
        h1 {{
            text-align: center;
            width: 100%;
        }}
        h2 {{
            display: flex;
            align-items: center;
            gap: 10px;
            font-size: 18px;
        }}
        h2 a {{
            margin-right: 10px;
        }}
        button {{
            font-weight: bold;
            font-size: 24px;
            padding: 8px 12px;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            transition: scale 0.15s;
            box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
        }}
        button:hover {{
            scale: 1.05;;
        }}
        button:active {{
            scale: 0.8;
        }}
        code {{
            background-color: #f5f5f5;
            padding: 5px 10px;
            border-radius: 4px;
            font-family: "Courier New", monospace;
        }}
        ul {{
            list-style-type: none;
            padding: 0;
            margin: 0;
            display: flex;
            flex-wrap: wrap;
            gap: 20px;
        }}
        li {{
            display: flex;
            flex-direction: column;
            align-items: center;
            padding: 20px;
            width: 150px;
            height: 150px;
            border: 1px solid #ddd;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
            transition: transform 0.2s;
            justify-content: center;
        }}
        li:hover {{
            transform: scale(1.05);
        }}
        a {{
            text-decoration: none;
            color: #333;
            display: flex;
            flex-direction: column;
            align-items: center;
        }}
        span:first-child {{
            margin-bottom: 10px;
            font-size: 36px;
        }}
        span:last-child {{
            font-size: 16px;
            font-weight: 500;
            color: #333;
            text-align: center;
            word-wrap: break-word;
        }}
        </style>
    </head>
    <body>
        {body}
    </body>
    </html>"#)
}

pub fn generate_ul(items: &Vec<String>) -> String {
    let mut li = String::new();
    for item in items {
        li.push_str(&format!("{item}"));
    };
    format!("<ul>{li}</ul>")
}