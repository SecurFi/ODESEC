"use client";

import type { NextPage } from "next";
import { Address } from "~~/components/scaffold-eth";
import { Card, Col, Empty, Input, List, Row } from 'antd';
import { useEffect, useState } from "react";
import { useScaffoldReadContract } from "~~/hooks/scaffold-eth/useScaffoldReadContract";
import { useScaffoldContract } from "~~/hooks/scaffold-eth/useScaffoldContract";

const { Search } = Input;

interface ProjectData {
  domain: string;
  contact: string;
  contracts: string[];
  owner: string;
}

const Home: NextPage = () => {
  // const { address: connectedAddress } = useAccount();
  const [projects, setProjects] = useState<ProjectData[]>([]);
  const [filteredProjects, setFilteredProjects] = useState<ProjectData[]>([]);
  const [searchValue, setSearchValue] = useState<string>("");
  const { data: totalProjects } = useScaffoldReadContract({
    contractName: "ODESEC",
    functionName: "totalProjects",
  });
  const { data: ODESEC } = useScaffoldContract({
    contractName: "ODESEC",
  });

  const updateProjects = async () => {
    if (!ODESEC) return;
    const data = await ODESEC!.read.getProjectList([BigInt(50), BigInt(projects.length)]);
    const newProjects = (data as any[]).map((project: any) => {
      console.log(project);
      return {
        domain: project.domain,
        contact: project.contact,
        contracts: project.contracts,
        owner: project.owner
      };
    });
    setProjects([...projects, ...newProjects]);
  };
  useEffect(() => {
    console.log('update: ', totalProjects);
    updateProjects();
  }, [totalProjects]);

  const onSearch = (value: string) => {
    console.log('projects: ', projects, value)
    if (!value) {
      setFilteredProjects([]);
      setSearchValue("");
      return;
    }
    const filtered = projects.filter(project => {
      console.log('project: ', project, value)
      if (project.domain.includes(value)) { return true }
      for (const contract of project.contracts) {
        if (contract.toLowerCase() == value.toLowerCase()) { return true }
      }
    });
    console.log('filtered: ', projects, filtered, value)
    setFilteredProjects(filtered);
    setSearchValue(value);

  }
  return (
    <>
      <div className="flex items-center flex-col flex-grow pt-10">
        <div className="px-5">
          <h1 className="text-center">
            <span className="block text-2xl mb-2">Welcome to</span>
            <span className="block text-4xl font-bold">ODESEC</span>
          </h1>
          <div className="flex justify-center items-center space-x-2">
            <p className="my-2 font-medium">On-chain Database of Emergency Security Event Contact, Whitehat is able to quickly establish a connection with the protocol</p>
          </div>

        </div>

        <div className="flex-grow bg-base-300 w-full mt-16 px-8 py-12">
          <div className="flex justify-center items-center gap-12 flex-col sm:flex-row">
            <Search
              className="max-w-[800px]"
              placeholder="Search by contract address or domain name"
              allowClear
              enterButton="Search"
              size="large"
              onSearch={onSearch}
            />
            <div className="stats shadow min-w-[150px]">
              <div className="stat">
                <div className="stat-title">Total Projects</div>
                <div className="stat-value">{totalProjects?.toString()}</div>
              </div>
            </div>
          </div>

          <Row gutter={16}>
            {filteredProjects.map((project, index) => (
              <Col span={8} key={index}>
                <Card className="min-w-[380px]" title={project.domain} extra={<Address address={project.owner} />}>
                  <div className="max-h-[300px] overflow-y-auto">
                    <p>Contact: {project.contact}</p>
                    <p>Contracts:</p>
                    <List
                      size="small"
                      dataSource={project.contracts}
                      renderItem={item => <List.Item>{item}</List.Item>}
                    />
                  </div>
                </Card>
              </Col>
            ))}
          </Row>
          {searchValue != "" && filteredProjects.length === 0 && (
            <Empty />
          )}
        </div>
      </div>
    </>
  );
};

export default Home;
