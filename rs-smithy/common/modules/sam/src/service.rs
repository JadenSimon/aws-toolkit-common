/**
 *     println!("Creating SAM service...");
    let mut service = service::Service::create().await.expect("Unable to start service");

    println!("Ready!");

    loop {
        let mut bytes: [u8; 10000] = [0; 10000]; // Bleh

        let input = tokio::io::stdin().read(&mut bytes).await.expect("Failed to read from stdin");
        let parsed = std::str::from_utf8(&bytes[..input]).expect("Failed to parse from stdin").trim();
        let request = serde_json::de::from_str::<aws_smithy_json_rpc::request::Request>(&parsed).expect("Failed to deserialize");


        let output = match request.method.as_str() {
            "$/aws/sam/pipeline/create/questions/answer" => {
                let params = serde_json::from_value::<toolkits::input::AnswerQuestionInput>(request.params.unwrap()).unwrap();
                serde_json::to_value(service.handle_answer_question_request(params).unwrap()).unwrap()
            },
            "$/aws/sam/pipeline/create/questions/list" => {
                let params = serde_json::from_value::<toolkits::input::ListQuestionsInput>(request.params.unwrap()).unwrap();
                serde_json::to_value(service.handle_list_questions_request(params).unwrap()).unwrap()
            },
            "$/aws/sam/pipeline/templates/list" => {
                let params = serde_json::from_value::<toolkits::input::ListPipelineTemplatesInput>(request.params.unwrap()).unwrap();
                serde_json::to_value(service.handle_list_pipeline_template_request(params).unwrap()).unwrap()
            },
            "$/aws/sam/pipeline/create" => {
                let params = serde_json::from_value::<toolkits::input::CreatePipelineTemplateFlowInput>(request.params.unwrap()).unwrap();
                serde_json::to_value(service.handle_create_pipeline_template_flow_request(params).unwrap()).unwrap()
            },
            _ => {
                // do nothing
                serde_json::Value::Null
            }
        };

        let response = aws_smithy_json_rpc::request::RequestBuilder::new()
            .id(request.id.unwrap())
            .version("2.0")
            .method(request.method)
            .params(serde_json::to_value(output).unwrap())
            .build()
            .unwrap();

        let mut writer = tokio::io::stdout();
        writer.write_all(&serde_json::to_string(&response).unwrap().as_bytes()).await.unwrap();
        writer.write("\n".as_bytes()).await.unwrap();
        writer.flush().await.unwrap();
    }

 */