"use client";
import type { NextPage } from "next";
import { useAccount } from "wagmi";
import { useState } from "react";
import { useScaffoldContract } from "~~/hooks/scaffold-eth/useScaffoldContract";
import { Result, Space, Steps } from 'antd';
import { Button, Form, Input } from 'antd';
import { DeleteOutlined } from '@ant-design/icons';
import Title from "antd/es/typography/Title";
import TextArea from "antd/es/input/TextArea";
import { useScaffoldWriteContract } from "~~/hooks/scaffold-eth/useScaffoldWriteContract";
import { TransactionReceipt } from "viem";
import { useRouter } from "next/navigation";

const ADDRESS_PATTERN = /^(0x)?[0-9a-fA-F]{40}$/

const Register: NextPage = () => {
  const { address: connectedAddress } = useAccount();
  const [step, setStep] = useState(0);
  const [challenge, setChallenge] = useState("");
  const router = useRouter();
  const [form] = Form.useForm();
  const ownerAddress = Form.useWatch("owner", { form, preserve: true })
  const [proof, setProof] = useState("");
  const [receipt, setReceipt] = useState<TransactionReceipt | null>(null);
  const { data: ODESEC } = useScaffoldContract({
    contractName: "ODESEC",
  });

  const { writeContractAsync, isPending } = useScaffoldWriteContract("ODESEC");

  const onStep1Finish = async (values: any) => {
    console.debug('Received values of form:', values);
    const data = await ODESEC!.read.makeChallenge([values.domain, values.owner]);
    console.debug(data);
    setChallenge(data.slice(2) + "." + values.domain);
    setStep(step + 1);
  }

  const handleSubmit = async () => {

    try {
      const p = proof.startsWith("0x") ? proof : "0x" + proof;
      await writeContractAsync({
        functionName: "addProject",
        args: [
          form.getFieldValue('domain'),
          form.getFieldValue('contact'),
          form.getFieldValue('contracts'),
          form.getFieldValue('owner'),
          p as any,
        ],
      }, {
        onBlockConfirmation: txnReceipt => {
          setReceipt(txnReceipt);
          console.log("📦 Transaction blockHash", txnReceipt.blockHash);
        },
      })
    } catch (error) {
      console.error("Error add project", error);
    }

  }

  return (
    <>
      {receipt == null && <div className="flex items-center flex-col flex-grow pt-10">
        <div className="flex flex-col px-5 items-center">

          <h1 className="text-center">
            <span className="block text-2xl mb-2">Project Register</span>
          </h1>
          <div className="text-center">
            <Steps
              // size="small"
              current={step}
              labelPlacement="vertical"
              items={[
                {
                  title: 'Step',
                },
                {
                  title: 'Obtain SSL certificates',
                },
                {
                  title: 'Generate SNARK Proof',
                },
                {
                  title: 'Submit'
                }
              ]}
            />
          </div>
          <div className="flex flex-col gap-4 mt-4">
            {step == 0 && <div>
              <Form
                className="w-[700px]"
                // style={{ maxWidth: 600 }}
                // layout="vertical"
                labelCol={{ span: 8 }}
                wrapperCol={{ span: 16 }}
                labelAlign="left"
                size="large"
                onFinish={onStep1Finish}
                initialValues={{
                  owner: ''
                }}
                form={form}>
                <Form.Item label="Project Domain" name="domain" rules={[{ required: true }]}>
                  <Input placeholder="uniswap.org" />
                </Form.Item>
                <Form.Item label="Owner Address" name="owner" rules={[{ required: true, message: 'Invalid Address' }]}>
                  <Space.Compact direction="horizontal" style={{ width: '100%' }}>
                    <Input value={ownerAddress} placeholder="0x6C9FC64A53c1b71FB3f9Af64d1ae3A4931A5f4E9" />
                    <Button style={{ width: 120 }} onClick={() => { form.setFieldValue('owner', connectedAddress) }} disabled={!connectedAddress}>My Address</Button>
                  </Space.Compact>
                </Form.Item>
                <Form.Item label="Contact Info" name="contact" rules={[{ required: true }]}>
                  <Input placeholder="tg:bot_name/chat_id" />
                </Form.Item>
                <Form.Item label="Contracts">

                  <Form.List name="contracts">
                    {(fields, opt) => (
                      <div style={{ display: 'flex', flexDirection: 'column', rowGap: 8 }}>
                        {fields.map((field) => (
                          <Space.Compact direction="horizontal" style={{ width: '100%' }} key={field.key}>
                            <Form.Item style={{ width: '100%' }} name={field.name} rules={[{ pattern: ADDRESS_PATTERN, message: 'Invalid Address' }]}>
                              <Input />
                            </Form.Item>
                            <Button danger onClick={() => { opt.remove(field.name) }} icon={<DeleteOutlined />} />

                          </Space.Compact>
                        ))}
                        <Button type="dashed" onClick={() => opt.add()} block>
                          + Add new Contract Address
                        </Button>
                      </div>
                    )}
                  </Form.List>
                </Form.Item>
                <Form.Item wrapperCol={{ xs: { span: 24, offset: 0 }, sm: { span: 16, offset: 8 } }}>
                  <Button type="primary" htmlType="submit">Generate Challenge</Button>
                </Form.Item>
              </Form>

            </div>}

            {step == 1 && <div>
              <div>
                <p>Use Let&apos;s Encrypt to generate a certificate for the domain <b>{challenge}</b></p>
                <Title level={4}>Use <a className="text-blue-600" href="https://github.com/srvrco/getssl">getsll</a> to obtain SSL certificates</Title>
                <div className="mockup-code text-left w-full mb-4">
                  <pre data-prefix="$"><code>curl --silent https://raw.githubusercontent.com/srvrco/getssl/latest/getssl {'>'} getssl ; chmod 700 getssl</code></pre>
                  <pre data-prefix="$"><code>./getssl -c {challenge}</code></pre>
                  <pre data-prefix="$"><code>./getssl {challenge}</code></pre>
                  <pre data-prefix="$"><code>#the certificate is {challenge}.crt</code></pre>
                </div>
                <Space>
                  <Button size="large" onClick={() => { setStep(step - 1) }}>Back</Button>
                  <Button size="large" onClick={() => { setStep(step + 1) }} type="primary">Next</Button>
                </Space>
              </div>
            </div>}

            {step == 2 && <div>
              <div>
                <Title level={4}>use the <a className="text-blue-600" href="https://github.com/SecurFi/ODESEC">odesec</a> tool to generate the snark proof of certificate
                </Title>
                <div className="mockup-code text-left w-full mb-4">
                  <pre data-prefix="$"><code>git clone https://github.com/SecurFi/ODESEC.git</code></pre>
                  <pre data-prefix="$"><code>cd ODESEC</code></pre>
                  <pre data-prefix="$"><code>export BONSAI_API_URL=https://api.bonsai.xyz/</code></pre>
                  <pre data-prefix="$"><code>export BONSAI_API_KEY=your-api-key </code></pre>
                  <pre data-prefix="$"><code>cargo run -- cert -p -c {challenge}.crt</code></pre>
                  <pre data-prefix="$"><code>#Copy the proof from the output for the next step.</code></pre>
                </div>
                <Space>
                  <Button size="large" onClick={() => { setStep(step - 1) }}>Back</Button>
                  <Button size="large" onClick={() => { setStep(step + 1) }} type="primary">Next</Button>
                </Space>
              </div>
            </div>}

            {step == 3 && <div>
              <div className="flex flex-col  align-middle w-[700px] mb-[24px]">
                <div className="mb-4">
                  <TextArea value={proof} onChange={(e) => setProof(e.target.value)} autoSize={{ minRows: 5, maxRows: 7 }} placeholder="Paste your SNARK Proof here" />
                </div>
                <Space>
                  <Button size="large" onClick={() => { setStep(step - 1) }}>Back</Button>
                  <Button loading={isPending} type="primary" size="large" onClick={handleSubmit} >Submit</Button>
                </Space>
              </div>
            </div>}
          </div>

        </div>

      </div>}

      {receipt != null && <div>
        <Result
          status="success"
          title="Successfully Register Your Project!"
          subTitle={`Tx hash: ${receipt.transactionHash}`}
          extra={[
            <Button type="primary" key="console" onClick={() => router.push("/")}>
              Go Home
            </Button>
          ]}
        />
      </div>}
    </>
  );
};

export default Register;
