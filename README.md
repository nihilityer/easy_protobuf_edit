# easy_protobuf_edit

简单的Protobuf数据阅读和修改

simple Protobuf data reading and modification

## 使用 / Usage:

### 1、获取file_descriptor_set / Get file_descriptor_set

```shell
$ protoc --proto_path="." -o file_descriptor_set.bin example.proto
```

### 2、使用GUI界面编辑 / Use the GUI for editing

```shell
$ easy_protobuf_edit
```

## 路线图 / Roadmap

- [x] Protobuf数据文件解析
- [ ] 更好用的页面交互
- [ ] 通过Json文件输出Protobuf Base64编码数据
- [ ] 可视化网页
- [ ] 更多功能……